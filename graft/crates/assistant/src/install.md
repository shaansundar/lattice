# crates/assistant/src/install.rs

- with_session · function · L12-L17 — pub fn with_session<R>(session: &mut Session, f: impl FnOnce(&mut dyn ToolHost) -> R) -> R
- server_file · function · L20-L22 — pub fn server_file() -> &'static str
- search_dirs · function · L26-L36 — pub fn search_dirs() -> Vec<PathBuf>
- locate · function · L39-L56 — pub fn locate(tier: Tier) -> Result<(PathBuf, PathBuf, Tier), String>
- free_port · function · L59-L61 — pub fn free_port() -> Option<u16>
- tests · module · L64-L84 — mod tests
- with_session_keeps_the_session · function · L69-L74 — fn with_session_keeps_the_session()
- missing_assets_explain_themselves · function · L77-L83 — fn missing_assets_explain_themselves()
