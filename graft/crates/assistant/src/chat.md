# crates/assistant/src/chat.rs

- BOS · constant · L11-L11 — pub const BOS: &str = "<bos>";
- role · function · L13-L15 — fn role(m: &Value) -> &str
- dictsort · function · L18-L22 — fn dictsort(o: &serde_json::Map<String, Value>) -> Vec<(&String, &Value)>
- format_argument · function · L25-L40 — pub fn format_argument(v: &Value, escape_keys: bool) -> String
- strip_thinking · function · L43-L52 — fn strip_thinking(text: &str) -> String
- content_text · function · L54-L73 — fn content_text(m: &Value, model: bool) -> String
- tool_response_block · function · L75-L81 — fn tool_response_block(name: &str, body: &Value) -> String
- Prev · enum · L84-L89 — enum Prev
- render · function · L92-L203 — pub fn render(messages: &[Value], enable_thinking: bool, add_generation_prompt: bool) -> String
