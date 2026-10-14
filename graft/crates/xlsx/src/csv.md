# crates/xlsx/src/csv.rs

- CP1252 · constant · L12-L15 — const CP1252: [u32; 32] = [
- decode_text · function · L18-L42 — pub fn decode_text(bytes: &[u8]) -> String
- sniff_delimiter · function · L46-L85 — pub fn sniff_delimiter(text: &str, quote: char) -> u8
- parse_records · function · L88-L131 — fn parse_records(text: &str, delim: char, quote: char, mut on_record: impl FnMut(Vec<String>) -> bool)
- read_csv · function · L133-L184 — pub fn read_csv(bytes: &[u8], opts: &CsvOptions) -> Result<Workbook, IoError>
- field_text · function · L186-L195 — fn field_text(v: &Value) -> String
- write_csv · function · L198-L220 — pub fn write_csv(sheet: &Sheet, wb: &Workbook, delimiter: u8) -> Vec<u8>
