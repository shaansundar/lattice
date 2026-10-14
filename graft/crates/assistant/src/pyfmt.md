# crates/assistant/src/pyfmt.rs

- float_repr · function · L10-L45 — pub fn float_repr(f: f64) -> String
- round6 · function · L48-L50 — pub fn round6(f: f64) -> f64
- py_str · function · L53-L61 — pub fn py_str(v: &Value) -> String
- number_str · function · L63-L71 — fn number_str(n: &serde_json::Number) -> String
- py_repr · function · L74-L102 — pub fn py_repr(v: &Value) -> String
- json_dumps · function · L106-L110 — pub fn json_dumps(v: &Value, compact: bool) -> String
- write_json · function · L112-L155 — fn write_json(out: &mut String, v: &Value, compact: bool)
- write_json_str · function · L157-L178 — fn write_json_str(out: &mut String, s: &str)
- truncate · function · L181-L188 — pub fn truncate(s: &str, max: usize) -> String
- tests · module · L191-L225 — mod tests
- float_repr_matches_python · function · L196-L214 — fn float_repr_matches_python()
- json_dumps_matches_python_ensure_ascii · function · L217-L224 — fn json_dumps_matches_python_ensure_ascii()
