# xtask/src/parity.rs

- Entry · struct · L14-L20 — pub struct Entry
- pct · function · L22-L24 — fn pct(done: usize, total: usize) -> f64
- render · function · L28-L71 — pub fn render(entries: &[Entry]) -> String
- entries · function · L74-L86 — fn entries() -> Vec<Entry>
- summary_line · function · L89-L92 — fn summary_line() -> String
- run · function · L95-L121 — pub fn run(root: &Path, args: &[&str]) -> Result<(), String>
- run · function · L124-L126 — pub fn run(_root: &Path, _args: &[&str]) -> Result<(), String>
- tests · module · L129-L152 — mod tests
- e · function · L132-L134 — fn e(id: &str, tab: &str, done: bool) -> Entry
- groups_by_tab_and_counts_unique_ids · function · L137-L146 — fn groups_by_tab_and_counts_unique_ids()
- empty_catalog_renders · function · L149-L151 — fn empty_catalog_renders()
