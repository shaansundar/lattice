//! Where the bundled runtime and models live, and how the app hands its workbook session to the harness.

use std::path::{Path, PathBuf};

use lattice_engine::Session;

use crate::harness::ToolHost;
use crate::router::Tier;

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

/// Directories searched for the assistant's files, in order: `LATTICE_ASSISTANT_DIR`, `assistant/` next to the
/// executable, and the macOS bundle's `Resources/assistant/`.
pub fn search_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(d) = std::env::var_os("LATTICE_ASSISTANT_DIR") {
        dirs.push(PathBuf::from(d));
    }
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|e| e.parent().map(Path::to_path_buf)) {
        dirs.push(exe_dir.join("assistant"));
        dirs.push(exe_dir.join("../Resources/assistant"));
    }
    dirs
}

/// (runtime binary, model file) for `tier`, falling back to the other tier's model if only that one is installed.
pub fn locate(tier: Tier) -> Result<(PathBuf, PathBuf, Tier), String> {
    let other = if tier == Tier::E4B { Tier::E2B } else { Tier::E4B };
    for dir in search_dirs() {
        let server = dir.join(server_file());
        if !server.exists() {
            continue;
        }
        for t in [tier, other] {
            let model = dir.join(t.model_file());
            if model.exists() {
                return Ok((server, model, t));
            }
        }
    }
    Err("The assistant's model isn't installed in this build. Desktop releases include it; for a source build, put \
         llama-server and the model in an assistant folder next to the app or set LATTICE_ASSISTANT_DIR."
        .into())
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
            assert!(e.contains("isn't installed"));
        }
        assert!(free_port().is_some());
    }
}
