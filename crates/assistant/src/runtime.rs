//! Local model runtime: a llama.cpp `llama-server` sidecar process on a loopback port, driven with a small
//! hand-written HTTP/1.1 client (no async runtime, no new dependencies).
//!
//! A sidecar keeps model faults out of the app's process (never-crash): if the server dies, the assistant reports
//! an error and can restart it; the workbook is untouched. The prompt is rendered by Lattice ([`crate::chat`]) and
//! sent to `/completion` as raw text with `cache_prompt`, so the fixed system prefix is computed once per session.

use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
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

/// Lines of the server's log kept for error messages.
const LOG_LINES: usize = 60;

/// Why the sidecar didn't start, in terms the next attempt can act on.
#[derive(Debug, Clone, PartialEq)]
pub enum StartError {
    /// The binary or model is missing or can't be executed: retrying won't help.
    Setup(String),
    /// Out of memory while loading (a smaller context or CPU-only may fit).
    Memory(String),
    /// The GPU backend failed (CPU-only may work).
    Gpu(String),
    /// Anything else (crash, timeout, port taken).
    Other(String),
}

impl StartError {
    pub fn message(&self) -> &str {
        match self {
            StartError::Setup(m) | StartError::Memory(m) | StartError::Gpu(m) | StartError::Other(m) => m,
        }
    }
}

pub struct LlamaServer {
    child: Mutex<Child>,
    addr: SocketAddr,
    log: Arc<Mutex<VecDeque<String>>>,
    pid_file: Option<PathBuf>,
}

impl LlamaServer {
    /// Start the sidecar and wait until it has loaded the model. `pid_file` records the process so a later launch
    /// can clean it up if the app is killed before it could.
    pub fn spawn(cfg: &RuntimeConfig, ready_timeout: Duration) -> Result<Self, String> {
        Self::start(cfg, ready_timeout, None).map_err(|e| e.message().to_string())
    }

    pub fn start(cfg: &RuntimeConfig, ready_timeout: Duration, pid_file: Option<PathBuf>) -> Result<Self, StartError> {
        if !cfg.server.is_file() {
            return Err(StartError::Setup(format!("The assistant's runtime is missing ({}). Reinstall Lattice.", cfg.server.display())));
        }
        if !cfg.model.is_file() {
            return Err(StartError::Setup(format!("The assistant's model is missing ({}).", cfg.model.display())));
        }
        let mut cmd = Command::new(&cfg.server);
        cmd.arg("-m").arg(&cfg.model).args(["--host", "127.0.0.1", "--port", &cfg.port.to_string()]);
        cmd.args(["--ctx-size", &cfg.ctx_size.to_string(), "--n-gpu-layers", &cfg.gpu_layers.to_string(), "--parallel", "1", "--no-webui"]);
        if cfg.threads > 0 {
            cmd.args(["--threads", &cfg.threads.to_string()]);
        }
        // Libraries ship next to the binary (llama.cpp's release layout).
        if let Some(dir) = cfg.server.parent() {
            cmd.current_dir(dir);
        }
        let mut child = cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::piped()).spawn().map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                StartError::Setup(format!("The assistant's runtime isn't allowed to run ({e}). Reinstall Lattice."))
            } else {
                StartError::Setup(format!("Can't start the assistant's runtime: {e}"))
            }
        })?;
        let log = Arc::new(Mutex::new(VecDeque::new()));
        if let Some(stderr) = child.stderr.take() {
            let log = Arc::clone(&log);
            std::thread::spawn(move || {
                for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                    if let Ok(mut l) = log.lock() {
                        if l.len() >= LOG_LINES {
                            l.pop_front();
                        }
                        l.push_back(line.chars().take(400).collect());
                    }
                }
            });
        }
        if let Some(p) = &pid_file {
            let _ = std::fs::write(p, child.id().to_string());
        }
        let addr = SocketAddr::from(([127, 0, 0, 1], cfg.port));
        let server = LlamaServer { child: Mutex::new(child), addr, log, pid_file };
        // Poll in fixed steps rather than reading the clock (Lattice's lint: clocks trap on wasm).
        const STEP: Duration = Duration::from_millis(250);
        let attempts = (ready_timeout.as_millis() / STEP.as_millis()).max(1);
        for _ in 0..attempts {
            if let Some(status) = server.exit_status() {
                // Give the log reader a moment to drain the last lines.
                std::thread::sleep(Duration::from_millis(100));
                return Err(classify(&server.log_tail(), &format!("the assistant's runtime stopped while loading ({status})")));
            }
            if let Ok((200, _)) = server.request("GET", "/health", None, &mut |_| true) {
                return Ok(server);
            }
            std::thread::sleep(STEP);
        }
        server.stop();
        Err(StartError::Other("The assistant's model took too long to load.".into()))
    }

    fn exit_status(&self) -> Option<std::process::ExitStatus> {
        self.child.lock().ok().and_then(|mut c| c.try_wait().ok().flatten())
    }

    pub fn is_running(&self) -> bool {
        self.child.lock().map(|mut c| matches!(c.try_wait(), Ok(None))).unwrap_or(false)
    }

    /// The last lines the server logged (for error reports).
    pub fn log_tail(&self) -> Vec<String> {
        self.log.lock().map(|l| l.iter().cloned().collect()).unwrap_or_default()
    }

    pub fn stop(&self) {
        if let Ok(mut c) = self.child.lock() {
            let _ = c.kill();
            let _ = c.wait();
        }
        if let Some(p) = &self.pid_file {
            let _ = std::fs::remove_file(p);
        }
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

/// Sort a failed start by what the server logged, so the launcher knows whether a smaller or CPU-only start can help.
pub fn classify(log: &[String], fallback: &str) -> StartError {
    let text = log.join("\n").to_ascii_lowercase();
    let last = log.iter().rev().find(|l| !l.trim().is_empty()).map(|l| format!(": {}", l.trim())).unwrap_or_default();
    if ["out of memory", "failed to allocate", "unable to allocate", "cannot allocate", "insufficient memory", "kiogpu"]
        .iter()
        .any(|k| text.contains(k))
    {
        StartError::Memory(format!("Not enough memory to load the assistant's model{last}"))
    } else if ["ggml_metal", "metal", "cuda", "vulkan", "gpu"].iter().any(|k| text.contains(k) && (text.contains("error") || text.contains("failed")))
    {
        StartError::Gpu(format!("The GPU couldn't run the assistant's model{last}"))
    } else if text.contains("failed to load model") || text.contains("invalid magic") || text.contains("unknown model architecture") {
        StartError::Setup(format!("The assistant's model file can't be loaded (it may be damaged){last}"))
    } else if text.contains("address already in use") || text.contains("couldn't bind") {
        StartError::Other(format!("The assistant's port was taken{last}"))
    } else {
        StartError::Other(format!("{fallback}{last}"))
    }
}

/// The pid file a Lattice process records its sidecar in: `llama-server-<app pid>.pid` in `dir`, one per app
/// process, so two running copies of Lattice never mistake each other's sidecar for a leftover.
pub fn pid_file_in(dir: &Path) -> PathBuf {
    dir.join(format!("llama-server-{}.pid", std::process::id()))
}

/// Stop sidecars left behind by Lattice processes that are gone (crashed or force-quit before they could stop
/// them). Files of live app processes are left alone, and only a process whose name is the runtime's is killed.
pub fn reap_stale(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        let Some(app) = name.strip_prefix("llama-server-").and_then(|s| s.strip_suffix(".pid")).and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        if app == std::process::id() || process_alive(app) {
            continue;
        }
        let content = std::fs::read_to_string(e.path()).ok();
        let _ = std::fs::remove_file(e.path());
        if let Some(pid) = content.and_then(|s| s.trim().parse::<u32>().ok())
            && pid != 0
            && pid != std::process::id()
            && process_name(pid).is_some_and(|n| n.to_ascii_lowercase().contains("llama-server"))
        {
            let _ = if cfg!(windows) {
                Command::new("taskkill").args(["/PID", &pid.to_string(), "/F"]).output()
            } else {
                Command::new("kill").arg(pid.to_string()).output()
            };
        }
    }
}

fn process_alive(pid: u32) -> bool {
    process_name(pid).is_some()
}

/// The executable name of a running process (`None` if there is no such process).
fn process_name(pid: u32) -> Option<String> {
    let out = if cfg!(windows) {
        Command::new("tasklist").args(["/FI", &format!("PID eq {pid}"), "/NH", "/FO", "CSV"]).output().ok()?
    } else {
        Command::new("ps").args(["-p", &pid.to_string(), "-o", "comm="]).output().ok()?
    };
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    // tasklist prints an INFO line (no quotes) when nothing matches.
    if text.is_empty() || (cfg!(windows) && !text.starts_with('"')) { None } else { Some(text) }
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
    fn start_failures_are_classified_from_the_log() {
        let l = |s: &str| s.lines().map(str::to_string).collect::<Vec<_>>();
        assert!(matches!(
            classify(&l("llama_model_load: loading\nggml_backend_cpu: failed to allocate buffer of size 9000 MiB"), "x"),
            StartError::Memory(_)
        ));
        assert!(matches!(classify(&l("ggml_metal_init: error: failed to create command queue"), "x"), StartError::Gpu(_)));
        assert!(matches!(classify(&l("gguf_init_from_file: invalid magic characters\nfailed to load model"), "x"), StartError::Setup(_)));
        assert!(matches!(classify(&l("couldn't bind HTTP server socket: address already in use"), "x"), StartError::Other(m) if m.contains("port")));
        let other = classify(&[], "stopped (signal 9)");
        assert_eq!(other, StartError::Other("stopped (signal 9)".into()));
    }

    #[test]
    #[cfg(unix)]
    fn stale_pid_files_never_touch_live_apps_or_other_processes() {
        let dir = std::env::temp_dir().join(format!("lattice-pid-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // A finished child gives a pid that belongs to no live process.
        let mut child = Command::new("true").spawn().or_else(|_| Command::new("cmd").args(["/C", "exit"]).spawn()).unwrap();
        child.wait().unwrap();
        let dead = child.id();
        // A live sidecar of ours that must survive: a `sleep` (not named llama-server, so never killable anyway).
        let mut live = Command::new("sleep").arg("30").spawn().unwrap();
        let mine = pid_file_in(&dir);
        std::fs::write(&mine, live.id().to_string()).unwrap();
        // Another live app (this test process's parent) and a dead one whose "sidecar" is our live `sleep`.
        let other_live = dir.join(format!("llama-server-{}.pid", std::os::unix::process::parent_id()));
        std::fs::write(&other_live, "1").unwrap();
        let stale = dir.join(format!("llama-server-{dead}.pid"));
        std::fs::write(&stale, live.id().to_string()).unwrap();
        let junk = dir.join("llama-server-notapid.pid");
        std::fs::write(&junk, "x").unwrap();
        reap_stale(&dir);
        assert!(mine.exists() && other_live.exists() && junk.exists(), "live apps' files and foreign files stay");
        assert!(!stale.exists(), "the dead app's file is cleaned up");
        assert!(matches!(live.try_wait(), Ok(None)), "a process not named llama-server is never killed");
        let _ = live.kill();
        let _ = live.wait();
        reap_stale(&dir.join("missing"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_runtime_is_an_error_not_a_crash() {
        let cfg =
            RuntimeConfig { server: "/nonexistent/llama-server".into(), model: "/x.gguf".into(), ctx_size: 8192, threads: 0, gpu_layers: 0, port: 1 };
        assert!(LlamaServer::spawn(&cfg, Duration::from_millis(10)).is_err());
    }
}
