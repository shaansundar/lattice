# crates/engine/src/sample.rs

- SAMPLES · constant · L11-L11 — pub const SAMPLES: &[(&str, &str)] = &[("budget", "Household Budget"), ("sales", "Quarterly Sales Dashboard"), ("grades", "Gradebook")];
- title · function · L13-L15 — pub fn title(name: &str) -> String
- build · function · L18-L45 — pub fn build(name: &str) -> Option<Workbook>
- run · function · L47-L55 — fn run(s: &mut Session, id: &str, p: serde_json::Value) -> bool
- sales · function · L57-L128 — fn sales(s: &mut Session) -> bool
- budget · function · L130-L173 — fn budget(s: &mut Session) -> bool
- grades · function · L175-L206 — fn grades(s: &mut Session) -> bool
- blank_sheet · function · L209-L211 — pub fn blank_sheet(name: &str) -> Sheet
