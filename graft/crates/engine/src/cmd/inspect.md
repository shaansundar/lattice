# crates/engine/src/cmd/inspect.rs

- specs · function · L8-L19 — pub fn specs() -> Vec<CommandSpec>
- value_json · function · L21-L34 — pub fn value_json(v: &Value) -> Json
- inspect · function · L36-L79 — fn inspect(s: &mut Session, _: &Json) -> Result<Json>
- read · function · L81-L118 — fn read(s: &mut Session, p: &Json) -> Result<Json>
- cell_get · function · L120-L142 — fn cell_get(s: &mut Session, p: &Json) -> Result<Json>
- sel_stats · function · L144-L149 — fn sel_stats(s: &mut Session, _: &Json) -> Result<Json>
- history_squash · function · L151-L161 — fn history_squash(s: &mut Session, p: &Json) -> Result<Json>
- history · function · L163-L168 — fn history(s: &mut Session, _: &Json) -> Result<Json>
- list_commands · function · L170-L179 — fn list_commands(s: &mut Session, p: &Json) -> Result<Json>
