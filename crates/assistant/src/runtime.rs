//! Local model runtime: a llama.cpp `llama-server` sidecar process on a loopback port, driven with a small
//! hand-written HTTP/1.1 client (no async runtime, no new dependencies).
//!
//! A sidecar keeps model faults out of the app's process (never-crash): if the server dies, the assistant reports
//! an error and can restart it; the workbook is untouched. The prompt is rendered by Lattice ([`crate::chat`]) and
//! sent to `/completion` as raw text with `cache_prompt`, so the fixed system prefix is computed once per session.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

use serde_json::{Value, json};

/// Limits on what the server may send back (hostile-input rule).
const MAX_HEADER_BYTES: usize = 64 * 1024;
const MAX_BODY_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// The `llama-server` binary shipped with the app.
    pub server: PathBuf,
    /// The GGUF model file for this machine's tier.
    pub model: PathBuf,
    pub ctx_size: u32,
    /// CPU threads for generation (0 = let llama.cpp decide).
    pub threads: u32,
    /// Layers offloaded to the GPU (0 = CPU only; 99 = all).
    pub gpu_layers: u32,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct GenParams {
    pub max_tokens: u32,
    pub temperature: f32,
    pub stop: Vec<String>,
}

impl Default for GenParams {
    fn default() -> Self {
        GenParams { max_tokens: 1536, temperature: 0.0, stop: vec!["<|tool_response>".into(), "<turn|>".into()] }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GenOutput {
    pub text: String,
    /// True when generation hit `max_tokens` (an unfinished answer or thought).
    pub truncated: bool,
    pub prompt_tokens: u64,
    pub generated_tokens: u64,
}

/// Produces text for a prompt; runs on a worker thread. `on_text` receives streamed pieces and returns false to stop.
pub trait Generator: Send + Sync {
    fn generate(&self, prompt: &str, params: &GenParams, on_text: &mut dyn FnMut(&str) -> bool) -> Result<GenOutput, String>;
}

pub struct LlamaServer {
    child: Child,
    addr: SocketAddr,
}

impl LlamaServer {
    /// Start the sidecar and wait until it has loaded the model.
    pub fn spawn(cfg: &RuntimeConfig, ready_timeout: Duration) -> Result<Self, String> {
        if !cfg.server.exists() {
            return Err(format!("assistant runtime not found at {}", cfg.server.display()));
        }
        if !cfg.model.exists() {
            return Err(format!("assistant model not found at {}", cfg.model.display()));
        }
        let mut cmd = Command::new(&cfg.server);
        cmd.arg("-m").arg(&cfg.model).args(["--host", "127.0.0.1", "--port", &cfg.port.to_string()]);
        cmd.args(["--ctx-size", &cfg.ctx_size.to_string(), "--n-gpu-layers", &cfg.gpu_layers.to_string(), "--parallel", "1", "--no-webui"]);
        if cfg.threads > 0 {
            cmd.args(["--threads", &cfg.threads.to_string()]);
        }
        let child = cmd
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("cannot start the assistant runtime: {e}"))?;
        let addr = SocketAddr::from(([127, 0, 0, 1], cfg.port));
        let mut server = LlamaServer { child, addr };
        // Poll in fixed steps rather than reading the clock (Lattice's lint: clocks trap on wasm).
        const STEP: Duration = Duration::from_millis(250);
        let attempts = (ready_timeout.as_millis() / STEP.as_millis()).max(1);
        for _ in 0..attempts {
            if let Ok(Some(status)) = server.child.try_wait() {
                return Err(format!("the assistant runtime exited while loading ({status})"));
            }
            if let Ok((200, _)) = server.request("GET", "/health", None, &mut |_| true) {
                return Ok(server);
            }
            std::thread::sleep(STEP);
        }
        server.stop();
        Err("the assistant model took too long to load".into())
    }

    pub fn is_running(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }

    pub fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    /// One HTTP request; streamed (chunked) bodies are handed to `on_body` piece by piece. Returns (status, body).
    fn request(&self, method: &str, path: &str, body: Option<&Value>, on_body: &mut dyn FnMut(&[u8]) -> bool) -> Result<(u16, Vec<u8>), String> {
        let mut s = TcpStream::connect_timeout(&self.addr, Duration::from_secs(2)).map_err(|e| e.to_string())?;
        s.set_read_timeout(Some(Duration::from_secs(600))).map_err(|e| e.to_string())?;
        let payload = body.map(|b| b.to_string()).unwrap_or_default();
        let head = format!(
            "{method} {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            payload.len()
        );
        s.write_all(head.as_bytes()).and_then(|_| s.write_all(payload.as_bytes())).map_err(|e| e.to_string())?;
        let mut r = BufReader::new(s);
        let (status, chunked, length) = read_head(&mut r)?;
        let mut all = Vec::new();
        if chunked {
            read_chunked(&mut r, &mut |piece| {
                if all.len() + piece.len() > MAX_BODY_BYTES {
                    return false;
                }
                all.extend_from_slice(piece);
                on_body(piece)
            })?;
        } else {
            let n = length.unwrap_or(MAX_BODY_BYTES).min(MAX_BODY_BYTES);
            r.take(n as u64).read_to_end(&mut all).map_err(|e| e.to_string())?;
            on_body(&all);
        }
        Ok((status, all))
    }
}

impl Drop for LlamaServer {
    fn drop(&mut self) {
        self.stop();
    }
}

fn read_head(r: &mut impl BufRead) -> Result<(u16, bool, Option<usize>), String> {
    let mut line = String::new();
    let mut total = 0usize;
    r.read_line(&mut line).map_err(|e| e.to_string())?;
    let status = line.split_whitespace().nth(1).and_then(|s| s.parse::<u16>().ok()).ok_or("malformed HTTP status line")?;
    let (mut chunked, mut length) = (false, None);
    loop {
        line.clear();
        let n = r.read_line(&mut line).map_err(|e| e.to_string())?;
        total += n;
        if n == 0 || total > MAX_HEADER_BYTES {
            return Err("malformed HTTP headers".into());
        }
        let l = line.trim_end();
        if l.is_empty() {
            break;
        }
        if let Some((k, v)) = l.split_once(':') {
            let (k, v) = (k.trim().to_ascii_lowercase(), v.trim());
            if k == "transfer-encoding" && v.to_ascii_lowercase().contains("chunked") {
                chunked = true;
            } else if k == "content-length" {
                length = v.parse().ok();
            }
        }
    }
    Ok((status, chunked, length))
}

fn read_chunked(r: &mut impl BufRead, on: &mut dyn FnMut(&[u8]) -> bool) -> Result<(), String> {
    let mut line = String::new();
    loop {
        line.clear();
        if r.read_line(&mut line).map_err(|e| e.to_string())? == 0 {
            return Ok(());
        }
        let size = usize::from_str_radix(line.trim().split(';').next().unwrap_or("0"), 16).map_err(|_| "malformed chunk size")?;
        if size == 0 {
            return Ok(());
        }
        if size > MAX_BODY_BYTES {
            return Err("chunk too large".into());
        }
        let mut buf = vec![0u8; size];
        r.read_exact(&mut buf).map_err(|e| e.to_string())?;
        let mut crlf = [0u8; 2];
        r.read_exact(&mut crlf).map_err(|e| e.to_string())?;
        if !on(&buf) {
            return Ok(());
        }
    }
}

/// Server-sent events: `data: {json}` lines, possibly split across chunks.
#[derive(Default)]
pub struct SseParser {
    buf: Vec<u8>,
}

impl SseParser {
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Value> {
        self.buf.extend_from_slice(bytes);
        let mut out = Vec::new();
        while let Some(pos) = self.buf.iter().position(|b| *b == b'\n') {
            let line: Vec<u8> = self.buf.drain(..=pos).collect();
            let text = String::from_utf8_lossy(&line);
            if let Some(data) = text.trim().strip_prefix("data:")
                && let Ok(v) = serde_json::from_str::<Value>(data.trim())
            {
                out.push(v);
            }
        }
        if self.buf.len() > MAX_BODY_BYTES {
            self.buf.clear();
        }
        out
    }
}

impl Generator for LlamaServer {
    fn generate(&self, prompt: &str, params: &GenParams, on_text: &mut dyn FnMut(&str) -> bool) -> Result<GenOutput, String> {
        let body = json!({
            "prompt": prompt, "n_predict": params.max_tokens, "temperature": params.temperature, "stop": params.stop,
            "stream": true, "cache_prompt": true, "special": true,
        });
        let mut sse = SseParser::default();
        let mut out = GenOutput { text: String::new(), truncated: false, prompt_tokens: 0, generated_tokens: 0 };
        let mut keep_going = true;
        let (status, raw) = self.request("POST", "/completion", Some(&body), &mut |piece| {
            for ev in sse.feed(piece) {
                if let Some(t) = ev.get("content").and_then(Value::as_str)
                    && !t.is_empty()
                {
                    out.text.push_str(t);
                    keep_going = on_text(t);
                }
                if ev.get("stop").and_then(Value::as_bool) == Some(true) {
                    out.truncated = ev.get("stop_type").and_then(Value::as_str) == Some("limit");
                    out.prompt_tokens = ev.get("tokens_evaluated").and_then(Value::as_u64).unwrap_or(0);
                    out.generated_tokens = ev.get("tokens_predicted").and_then(Value::as_u64).unwrap_or(0);
                }
            }
            keep_going
        })?;
        if status != 200 {
            return Err(format!("assistant runtime error {status}: {}", String::from_utf8_lossy(&raw).chars().take(300).collect::<String>()));
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sse_parser_handles_split_and_junk() {
        let mut p = SseParser::default();
        assert!(p.feed(b"data: {\"content\":\"he").is_empty());
        let evs = p.feed(b"llo\"}\n\ndata: not json\ndata: {\"stop\":true,\"stop_type\":\"limit\"}\n");
        assert_eq!(evs.len(), 2);
        assert_eq!(evs[0]["content"], "hello");
    }

    #[test]
    fn http_head_and_chunked_body() {
        let raw = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nhello\r\n6\r\n world\r\n0\r\n\r\n";
        let mut r = BufReader::new(&raw[..]);
        assert_eq!(read_head(&mut r).unwrap(), (200, true, None));
        let mut got = Vec::new();
        read_chunked(&mut r, &mut |b| {
            got.extend_from_slice(b);
            true
        })
        .unwrap();
        assert_eq!(got, b"hello world");
        assert!(read_head(&mut BufReader::new(&b"garbage"[..])).is_err());
        assert!(read_chunked(&mut BufReader::new(&b"zz\r\n"[..]), &mut |_| true).is_err());
    }

    #[test]
    fn missing_runtime_is_an_error_not_a_crash() {
        let cfg =
            RuntimeConfig { server: "/nonexistent/llama-server".into(), model: "/x.gguf".into(), ctx_size: 8192, threads: 0, gpu_layers: 0, port: 1 };
        assert!(LlamaServer::spawn(&cfg, Duration::from_millis(10)).is_err());
    }
}
