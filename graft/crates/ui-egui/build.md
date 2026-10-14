# crates/ui-egui/build.rs

- main · function · L15-L27 — fn main()
- read · function · L29-L36 — fn read<T>(path: &std::path::Path, fallback: &str, parse: impl Fn(&str) -> Result<T, String>) -> Option<T>
- name · function · L39-L44 — fn name(v: Option<&str>) -> String
- s · function · L46-L48 — fn s(v: &Value, k: &str) -> String
- n · function · L50-L52 — fn n(v: &Value, k: &str) -> u64
- rows · function · L54-L56 — fn rows(json: &Value, k: &str) -> Vec<Value>
- generate · function · L58-L105 — fn generate(json: &Value, people: &toml::Table) -> String
