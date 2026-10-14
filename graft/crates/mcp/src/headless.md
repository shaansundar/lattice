# crates/mcp/src/headless.rs

- NO_UI · constant · L9-L10 — pub(crate) const NO_UI: &str = "no UI in headless mode: start the app with `lattice --control 7979` and run the MCP server \
- Headless · struct · L13-L15 — pub struct Headless
- default · function · L18-L20 — fn default() -> Self
- new · function · L25-L29 — pub fn new() -> Self
- with_session · function · L32-L34 — pub fn with_session(session: Session) -> Self
- execute · function · L36-L39 — fn execute(&mut self, id: &str, params: Value) -> Result<Value, String>
- commands · function · L41-L57 — fn commands(&self, params: &Value) -> Value
- call · function · L61-L81 — fn call(&mut self, method: &str, params: Value) -> Result<Value, String>
- has_ui · function · L83-L85 — fn has_ui(&self) -> bool
- describe · function · L87-L89 — fn describe(&self) -> String
