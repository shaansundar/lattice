# crates/mcp/src/backend.rs

- Backend · interface · L11-L18 — pub trait Backend
- call · function · L13-L13 — fn call(&mut self, method: &str, params: Value) -> Result<Value, String>;
- has_ui · function · L15-L15 — fn has_ui(&self) -> bool;
- describe · function · L17-L17 — fn describe(&self) -> String;
- IO_TIMEOUT · constant · L21-L21 — const IO_TIMEOUT: Duration = Duration::from_secs(30);
- MAX_REPLY · constant · L23-L23 — const MAX_REPLY: u64 = 256 * 1024 * 1024;
- Remote · struct · L26-L30 — pub struct Remote
- connect · function · L34-L38 — pub fn connect(addr: &str) -> std::io::Result<Self>
- addr · function · L40-L42 — pub fn addr(&self) -> &str
- reconnect · function · L44-L61 — fn reconnect(&mut self) -> std::io::Result<()>
- roundtrip · function · L63-L79 — fn roundtrip(&mut self, line: &str) -> std::io::Result<String>
- call · function · L83-L104 — fn call(&mut self, method: &str, params: Value) -> Result<Value, String>
- has_ui · function · L106-L108 — fn has_ui(&self) -> bool
- describe · function · L110-L112 — fn describe(&self) -> String
