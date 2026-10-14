# crates/assistant/src/harness.rs

- ToolHost · interface · L11-L13 — pub trait ToolHost
- call · function · L12-L12 — fn call(&mut self, name: &str, args: &Value) -> Result<Value, String>;
- call · function · L17-L25 — fn call(&mut self, name: &str, args: &Value) -> Result<Value, String>
- MAX_CELLS_PER_WRITE · constant · L28-L28 — pub const MAX_CELLS_PER_WRITE: usize = 5000;
- is_risky · function · L31-L40 — pub fn is_risky(name: &str, args: &Value) -> bool
- Harness · struct · L42-L44 — pub struct Harness
- default · function · L47-L49 — fn default() -> Self
- run · function · L54-L98 — pub fn run(&self, host: &mut dyn ToolHost, name: &str, args: &Value, confirm: &mut dyn FnMut(&str, &Value) -> bool) -> String
- snapshot · function · L101-L118 — pub fn snapshot(&self, host: &mut dyn ToolHost) -> String
- short · function · L122-L124 — pub fn short(v: &Value) -> String
