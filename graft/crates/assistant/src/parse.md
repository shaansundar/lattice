# crates/assistant/src/parse.rs

- Q · constant · L7-L7 — const Q: &str = "<|\"|>";
- MAX_DEPTH · constant · L8-L8 — const MAX_DEPTH: usize = 32;
- ToolCall · struct · L11-L14 — pub struct ToolCall
- Cursor · struct · L16-L19 — struct Cursor<'a>
- rest · function · L22-L24 — fn rest(&self) -> &str
- peek · function · L25-L27 — fn peek(&self) -> Option<char>
- bump · function · L28-L32 — fn bump(&mut self)
- skip_ws · function · L33-L37 — fn skip_ws(&mut self)
- starts · function · L38-L40 — fn starts(&self, p: &str) -> bool
- parse_value · function · L43-L122 — fn parse_value(c: &mut Cursor, depth: usize) -> Result<Value, ()>
- scalar · function · L124-L141 — fn scalar(raw: &str) -> Value
- parse_object · function · L143-L190 — fn parse_object(c: &mut Cursor, depth: usize) -> Result<Value, ()>
- parse_tool_calls · function · L193-L203 — pub fn parse_tool_calls(text: &str) -> Vec<ToolCall>
- SPECIAL_STRIP · constant · L205-L205 — const SPECIAL_STRIP: [&str; 5] = ["<eos>", "<turn|>", "<end_of_turn>", "<|tool_response>", "<bos>"];
- clean_final · function · L207-L213 — fn clean_final(t: &str) -> String
- split_thinking · function · L216-L228 — pub fn split_thinking(t: &str) -> (String, String, bool)
- tests · module · L231-L244 — mod tests
- hostile_inputs_never_panic_or_hang · function · L235-L243 — fn hostile_inputs_never_panic_or_hang()
