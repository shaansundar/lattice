//! Where the bundled runtime and models live, and how the app hands its workbook session to the harness.

use std::path::{Path, PathBuf};
use std::time::Duration;

use lattice_engine::Session;

use crate::harness::ToolHost;
use crate::router::{Tier, gpu_layers};
use crate::runtime::{LlamaServer, RuntimeConfig, StartError};

/// Run `f` with a tool host over the app's live session (the session moves into lattice-mcp's in-process backend
/// for the call and back afterwards; nothing is copied).
pub fn with_session<R>(session: &mut Session, f: impl FnOnce(&mut dyn ToolHost) -> R) -> R {
    let mut host = lattice_mcp::Headless::with_session(std::mem::take(session));
    let r = f(&mut host);
    *session = host.session;
    r
}

/// The runtime binary's file name on this platform.
pub fn server_file() -> &'static str {
    if cfg!(windows) { "llama-server.exe" } else { "llama-server" }
}

/// Directories searched for the bundled runtime, in order: `LATTICE_ASSISTANT_DIR`, `assistant/` next to the
/// executable (Windows, Linux, source builds), and the macOS bundle's `Helpers/assistant/` (and the older
/// `Resources/assistant/`).
pub fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = std::env::var_os("LATTICE_ASSISTANT_DIR").filter(|d| !d.is_empty()) {
        dirs.push(PathBuf::from(d));
    }
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        dirs.push(exe_dir.join("assistant"));
        dirs.push(exe_dir.join("../Helpers/assistant"));
        dirs.push(exe_dir.join("../Resources/assistant"));
    }
    dirs
}

/// The bundled `llama-server`, if this build has one.
pub fn locate_server() -> Option<PathBuf> {
    search_dirs().into_iter().map(|d| d.join(server_file())).find(|p| p.is_file())
}

/// (runtime binary, model file, tier) to run: the bundled runtime and the best installed model (see
/// [`crate::models::choose`]). `tier` is preferred when it is installed.
pub fn locate(tier: Tier) -> Result<(PathBuf, PathBuf, Tier), String> {
    let server = locate_server().ok_or(MISSING_RUNTIME)?;
    if let Some(p) = crate::models::installed(tier) {
        return Ok((server, p, tier));
    }
    let (t, p) = crate::models::choose().ok_or("No assistant model is installed yet.")?;
    Ok((server, p, t))
}

/// Where this app process records its sidecar's process id (next to the models folder; one file per app process).
pub fn pid_file() -> Option<PathBuf> {
    crate::models::models_dir().and_then(|d| d.parent().map(crate::runtime::pid_file_in))
}

pub const MISSING_RUNTIME: &str = "This build of Lattice doesn't include the assistant's runtime (llama-server). Desktop releases \
     include it; for a source build, put llama.cpp's llama-server and its libraries in an assistant folder next to the app \
     or set LATTICE_ASSISTANT_DIR.";

/// A running sidecar and how it was started.
pub struct Launched {
    pub server: LlamaServer,
    pub tier: Tier,
    pub gpu_layers: u32,
    pub ctx_size: u32,
    /// What had to be given up to start (e.g. "running on the CPU: the GPU failed"), for the pane to mention.
    pub notes: Vec<String>,
}

/// Start the runtime for `model`, falling back when it fails: GPU → CPU, then a smaller context when memory runs
/// out, and a new port if the old one was taken. With `pid_file` (see [`pid_file`]), first cleans up a sidecar a
/// crashed earlier run left behind.
pub fn launch(server: &Path, model: &Path, tier: Tier, pid_file: Option<PathBuf>) -> Result<Launched, String> {
    if let Some(dir) = pid_file.as_deref().and_then(Path::parent) {
        crate::runtime::reap_stale(dir);
        let _ = std::fs::create_dir_all(dir);
    }
    // Context size barely moves memory with these models (E2B on CPU: 4.87 GB at 4k, 4.95 GB at 16k; it's the
    // weights), so every machine starts at 16k and only an out-of-memory failure shrinks it.
    let mut gpu = gpu_layers();
    let mut ctx: u32 = 16384;
    let mut notes = Vec::new();
    let mut last = String::new();
    for _ in 0..5 {
        let port = free_port().ok_or("no free local port for the assistant")?;
        let cfg = RuntimeConfig { server: server.into(), model: model.into(), ctx_size: ctx, threads: 0, gpu_layers: gpu, port };
        match LlamaServer::start(&cfg, Duration::from_secs(240), pid_file.clone()) {
            Ok(s) => return Ok(Launched { server: s, tier, gpu_layers: gpu, ctx_size: ctx, notes }),
            Err(StartError::Setup(m)) => return Err(m),
            Err(StartError::Gpu(m)) if gpu > 0 => {
                log::warn!("assistant: {m}; retrying on the CPU");
                notes.push("Running on the CPU because the GPU couldn't load the model.".into());
                gpu = 0;
                last = m;
            }
            Err(StartError::Memory(m)) if ctx > 4096 => {
                log::warn!("assistant: {m}; retrying with a smaller context");
                ctx /= 2;
                notes.push("Using a shorter memory for conversations to fit this computer.".into());
                last = m;
            }
            Err(StartError::Memory(m)) if gpu > 0 => {
                gpu = 0;
                last = m;
            }
            Err(StartError::Other(m)) => {
                log::warn!("assistant: {m}; retrying");
                last = m;
            }
            Err(e) => return Err(e.message().to_string()),
        }
    }
    Err(last)
}

/// A free loopback port for the runtime.
pub fn free_port() -> Option<u16> {
    std::net::TcpListener::bind(("127.0.0.1", 0)).ok().and_then(|l| l.local_addr().ok()).map(|a| a.port())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn with_session_keeps_the_session() {
        let mut s = Session::new();
        s.execute("file.new", json!({})).unwrap();
        with_session(&mut s, |h| h.call("set_cell", &json!({"cell": "A1", "input": "7"}))).unwrap();
        assert_eq!(s.execute("cell.get", json!({"cell": "A1"})).unwrap()["value"], json!(7.0));
    }

    #[test]
    fn missing_assets_explain_themselves() {
        let r = locate(Tier::E2B);
        if let Err(e) = r {
            assert!(e.contains("llama-server") || e.contains("No assistant model"), "{e}");
        }
        let e = launch(Path::new("/nonexistent/llama-server"), Path::new("/x.gguf"), Tier::E2B, None).err().unwrap_or_default();
        assert!(e.contains("missing"), "{e}");
        assert!(free_port().is_some());
    }
}
