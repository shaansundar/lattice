# crates/xlsx/src/tables.rs

- BUILTIN_FORMATS · constant · L6-L43 — const BUILTIN_FORMATS: [(u32, &str); 36] = [
- builtin_format · function · L45-L47 — pub fn builtin_format(id: u32) -> Option<&'static str>
- builtin_id · function · L51-L53 — pub fn builtin_id(code: &str) -> Option<u32>
- INDEXED · constant · L56-L65 — pub const INDEXED: [u32; 64] = [
- PAPERS · constant · L68-L83 — const PAPERS: [(u32, &str); 14] = [
- paper_name · function · L85-L87 — pub fn paper_name(id: u32) -> Option<&'static str>
- paper_id · function · L89-L91 — pub fn paper_id(name: &str) -> Option<u32>
- FUTURE_FUNCTIONS · constant · L95-L252 — const FUTURE_FUNCTIONS: &[&str] = &[
- file_function_name · function · L255-L267 — pub fn file_function_name(name: &str) -> Option<String>
- col_width_to_px · function · L272-L278 — pub fn col_width_to_px(w: f64) -> f32
- px_to_col_width · function · L281-L292 — pub fn px_to_col_width(px: f32) -> f64
- pt_to_px · function · L295-L297 — pub fn pt_to_px(pt: f64) -> f32
- px_to_pt · function · L299-L301 — pub fn px_to_pt(px: f32) -> f64
- EMU_PER_PX · constant · L304-L304 — pub const EMU_PER_PX: f64 = 9525.0;
- tests · module · L307-L332 — mod tests
- widths · function · L311-L319 — fn widths()
- functions · function · L322-L331 — fn functions()
