# crates/ui-egui/src/editor.rs

- REF_COLORS · constant · L8-L17 — pub const REF_COLORS: [Color32; 8] = [
- EditState · struct · L20-L44 — pub struct EditState
- new · function · L47-L66 — pub fn new(sheet: usize, cell: CellRef, text: String, enter_mode: bool, from_formula_bar: bool) -> EditState
- is_formula · function · L68-L72 — pub fn is_formula(&self) -> bool
- byte_caret · function · L74-L76 — fn byte_caret(&self) -> usize
- can_point · function · L80-L92 — pub fn can_point(&self) -> bool
- insert_ref · function · L95-L108 — pub fn insert_ref(&mut self, r: &str)
- update_autocomplete · function · L111-L129 — pub fn update_autocomplete(&mut self, names: &[String])
- accept_autocomplete · function · L132-L145 — pub fn accept_autocomplete(&mut self) -> bool
- current_function · function · L148-L193 — pub fn current_function(&self) -> Option<(String, usize)>
- formula_refs · function · L198-L239 — pub fn formula_refs(text: &str) -> Vec<(usize, usize, RangeRef, Option<String>, usize)>
- column_completion · function · L243-L282 — pub fn column_completion(sheet: &lattice_engine::model::Sheet, at: CellRef, typed: &str) -> Option<String>
- tests · module · L285-L324 — mod tests
- point_mode · function · L289-L302 — fn point_mode()
- autocomplete_and_hints · function · L305-L314 — fn autocomplete_and_hints()
- ref_spans · function · L317-L323 — fn ref_spans()
