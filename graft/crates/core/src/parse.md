# crates/core/src/parse.rs

- Parsed · struct · L12-L15 — pub struct Parsed
- parse_input · function · L19-L40 — pub fn parse_input(s: &str, sys: DateSystem) -> Parsed
- parse_number_text · function · L43-L45 — pub fn parse_number_text(s: &str) -> Option<f64>
- parse_numeric · function · L47-L65 — fn parse_numeric(t: &str, sys: DateSystem) -> Option<(f64, Option<&'static str>)>
- parse_plain · function · L68-L142 — fn parse_plain(t: &str) -> Option<(f64, Option<&'static str>)>
- parse_fraction · function · L145-L157 — fn parse_fraction(t: &str) -> Option<f64>
- month_from_name · function · L159-L165 — fn month_from_name(s: &str) -> Option<u32>
- parse_time · function · L168-L205 — fn parse_time(t: &str) -> Option<(f64, bool, bool)>
- parse_date · function · L207-L263 — fn parse_date(t: &str, sys: DateSystem) -> Option<(f64, &'static str)>
- valid_ymd · function · L265-L274 — fn valid_ymd(sys: DateSystem, y: i64, m: i64, d: i64) -> Option<f64>
- current_year · function · L279-L288 — pub fn current_year() -> i64
- parse_datetime · function · L290-L311 — fn parse_datetime(t: &str, sys: DateSystem) -> Option<(f64, Option<&'static str>)>
- tests · module · L314-L376 — mod tests
- p · function · L317-L319 — fn p(s: &str) -> Parsed
- numbers · function · L322-L335 — fn numbers()
- texts_and_specials · function · L338-L346 — fn texts_and_specials()
- dates_and_times · function · L349-L360 — fn dates_and_times()
- fictitious_1900_02_29 · function · L363-L375 — fn fictitious_1900_02_29()
