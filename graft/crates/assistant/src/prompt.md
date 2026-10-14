# crates/assistant/src/prompt.rs

- PREVIEW_ROWS · constant · L9-L9 — pub const PREVIEW_ROWS: u32 = 12;
- PREVIEW_COLS · constant · L10-L10 — pub const PREVIEW_COLS: u32 = 10;
- SYSTEM · constant · L13-L35 — pub const SYSTEM: &str = "You are the assistant built into Lattice, a spreadsheet app. You work on the user's open workbook with tools; your changes apply directly and the user can undo each request in one step.
- TOOL_NAMES · constant · L38-L54 — pub const TOOL_NAMES: [&str; 15] = [
- system_message · function · L56-L58 — pub fn system_message() -> Value
- preview_range · function · L61-L71 — pub fn preview_range(workbook: &Value) -> Option<(String, String)>
- context_block · function · L74-L81 — pub fn context_block(workbook: &Value, preview: Option<(&str, &str, &[Vec<Value>], &[Vec<Value>])>) -> String
- user_message · function · L83-L85 — pub fn user_message(context: &str, request: &str) -> Value
