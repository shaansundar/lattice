# crates/engine/src/tables.rs

- parse_style_name · function · L8-L17 — pub fn parse_style_name(name: &str) -> (&'static str, u32)
- accent_for · function · L20-L23 — fn accent_for(n: u32) -> Option<u8>
- TableLook · struct · L26-L34 — pub struct TableLook
- look · function · L36-L79 — pub fn look(style: &str) -> TableLook
- table_cell_style · function · L82-L134 — pub fn table_cell_style(wb: &Workbook, sheet: usize, t: &Table, c: CellRef) -> Style
- gallery · function · L137-L149 — pub fn gallery() -> Vec<String>
- sync_headers · function · L153-L245 — pub(crate) fn sync_headers(cx: &mut crate::cmd::Ctx)
- Names · type · L171-L171 — type Names = (std::collections::HashMap<String, usize>, std::collections::HashMap<String, usize>);
- tests · module · L248-L288 — mod tests
- editing_a_header_renames_the_column · function · L255-L278 — fn editing_a_header_renames_the_column()
- go_to_a_table_selects_its_data · function · L281-L287 — fn go_to_a_table_selects_its_data()
- style_tests · module · L291-L302 — mod style_tests
- zero_numbered_styles_do_not_underflow · function · L295-L301 — fn zero_numbered_styles_do_not_underflow()
