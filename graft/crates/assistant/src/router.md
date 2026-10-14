# crates/assistant/src/router.rs

- Entry · enum · L9-L14 — pub enum Entry
- HOW_TO · constant · L16-L16 — const HOW_TO: [&str; 9] = ["how do i", "how can i", "where is", "where's", "where do i", "what does", "is there a shortcut", "lattice-cli", "mcp"];
- MATH · constant · L17-L18 — const MATH: [&str; 10] =
- MULTI_STEP · constant · L19-L20 — const MULTI_STEP: [&str; 10] =
- use_thinking · function · L23-L32 — pub fn use_thinking(entry: Entry, request: &str) -> bool
- Tier · enum · L36-L41 — pub enum Tier
- model_file · function · L44-L49 — pub fn model_file(self) -> &'static str
- detect_tier · function · L54-L64 — pub fn detect_tier() -> Tier
- gpu_layers · function · L67-L72 — pub fn gpu_layers() -> u32
- total_ram_bytes · function · L74-L96 — fn total_ram_bytes() -> Option<u64>
- tests · module · L99-L112 — mod tests
- routes_quick_requests_fast_and_heavy_ones_to_thinking · function · L103-L111 — fn routes_quick_requests_fast_and_heavy_ones_to_thinking()
