# crates/engine/src/cmd/spelling.rs

- specs · function · L12-L19 — pub fn specs() -> Vec<CommandSpec>
- dictionary · function · L21-L36 — fn dictionary() -> &'static HashSet<String>
- DICT · constant · L22-L22 — static DICT: OnceLock<HashSet<String>> = OnceLock::new();
- known · function · L38-L59 — fn known(s: &Session, w: &str) -> bool
- edits1 · function · L61-L88 — fn edits1(w: &str) -> Vec<String>
- suggestions · function · L90-L109 — fn suggestions(w: &str) -> Vec<String>
- spelling · function · L111-L142 — fn spelling(s: &mut Session, p: &Json) -> Result<Json>
- add_word · function · L144-L150 — fn add_word(s: &mut Session, p: &Json) -> Result<Json>
- change · function · L152-L161 — fn change(s: &mut Session, p: &Json) -> Result<Json>
- thesaurus · function · L163-L173 — fn thesaurus(_: &mut Session, p: &Json) -> Result<Json>
