# crates/formula/src/adjust.rs

- shift_relative · function · L9-L17 — pub fn shift_relative(e: Expr, dr: i64, dc: i64) -> Expr
- shift_anchor · function · L19-L24 — fn shift_anchor(a: &Anchor, dr: i64, dc: i64) -> Option<Anchor>
- shift_line · function · L26-L32 — fn shift_line(v: u32, abs: bool, d: i64, max: u32) -> Option<u32>
- shift_ref · function · L34-L42 — fn shift_ref(r: &Reference, dr: i64, dc: i64) -> Option<Reference>
- Axis · enum · L45-L48 — pub enum Axis
- Edit · enum · L52-L67 — pub enum Edit
- adjust · function · L70-L88 — pub fn adjust(e: Expr, host: &str, target: &str, edit: &Edit) -> Expr
- map_index · function · L91-L106 — fn map_index(v: u32, at: u32, count: u32, insert: bool, max: u32) -> Option<u32>
- map_span · function · L109-L140 — fn map_span(lo: u32, hi: u32, at: u32, count: u32, insert: bool, max: u32) -> Option<(u32, u32)>
- adjust_ref · function · L142-L223 — fn adjust_ref(r: &Reference, edit: &Edit) -> Option<Reference>
- rename_sheet · function · L226-L252 — pub fn rename_sheet(e: Expr, old: &str, new: &str) -> Expr
- delete_sheet · function · L256-L278 — pub fn delete_sheet(e: Expr, name: &str, order: &[String]) -> Expr
