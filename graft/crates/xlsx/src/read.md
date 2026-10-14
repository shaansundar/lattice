# crates/xlsx/src/read.rs

- Ctx · struct · L14-L31 — pub struct Ctx<'a>
- warn · function · L34-L36 — pub fn warn(&mut self, m: impl Into<String>)
- next_object_id · function · L38-L42 — pub fn next_object_id(&mut self) -> u32
- style · function · L43-L45 — pub fn style(&self, xf: Option<u32>) -> StyleId
- optional_xml · function · L47-L60 — pub fn optional_xml(&mut self, part: &str) -> Result<Option<El>, IoError>
- find_rel · function · L63-L65 — pub fn find_rel<'r>(rels: &'r [Rel], kind: &str) -> Option<&'r Rel>
- read_xlsx · function · L67-L318 — pub fn read_xlsx(bytes: &[u8]) -> Result<(Workbook, ReadReport), IoError>
- unique_name · function · L320-L329 — pub(crate) fn unique_name(wb: &Workbook, name: &str, i: usize) -> String
- strip_sheet · function · L332-L337 — fn strip_sheet(s: &str) -> &str
- parse_print_area · function · L339-L343 — pub fn parse_print_area(text: &str) -> Option<RangeRef>
- parse_print_titles · function · L345-L362 — pub fn parse_print_titles(text: &str) -> (Option<(u32, u32)>, Option<(u32, u32)>)
