# crates/assistant/src/docs.rs

- MAX_CHARS · constant · L7-L7 — const MAX_CHARS: usize = 1400;
- TOP_K · constant · L8-L8 — const TOP_K: usize = 3;
- DOC_FILES · constant · L11-L21 — pub const DOC_FILES: &[(&str, &str)] = &[
- tokens · function · L24-L47 — pub fn tokens(text: &str) -> Vec<String>
- python_strip · function · L49-L51 — fn python_strip(s: &str) -> &str
- chunk_markdown · function · L54-L96 — pub fn chunk_markdown(name: &str, text: &str) -> Vec<(String, String)>
- DocsIndex · struct · L98-L103 — pub struct DocsIndex
- new · function · L106-L116 — pub fn new(chunks: Vec<(String, String)>) -> Self
- bundled · function · L119-L121 — pub fn bundled() -> Self
- len · function · L123-L125 — pub fn len(&self) -> usize
- is_empty · function · L127-L129 — pub fn is_empty(&self) -> bool
- search · function · L131-L155 — pub fn search(&self, query: &str) -> Vec<&(String, String)>
- render · function · L158-L164 — pub fn render(&self, query: &str) -> String
