# crates/xlsx/src/tests/mod.rs

- array_export · module · L3-L3 — mod array_export;
- csv_tests · module · L4-L4 — mod csv_tests;
- fixtures · module · L5-L5 — mod fixtures;
- malformed · module · L6-L6 — mod malformed;
- ods_tests · module · L7-L7 — mod ods_tests;
- pivot_tests · module · L8-L8 — mod pivot_tests;
- richdata_tests · module · L9-L9 — mod richdata_tests;
- roundtrip · module · L10-L10 — mod roundtrip;
- smooth_roundtrip · module · L11-L11 — mod smooth_roundtrip;
- xlsb_tests · module · L12-L12 — mod xlsb_tests;
- make_zip · function · L17-L24 — pub fn make_zip(parts: &[(&str, &[u8])]) -> Vec<u8>
- CT · constant · L26-L26 — pub const CT: &str = r#"<?xml version="1.0"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="xml" ContentType="application/xml"/><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/></Types>"#;
- ROOT_RELS · constant · L27-L27 — pub const ROOT_RELS: &str = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/></Relationships>"#;
- minimal · function · L31-L52 — pub fn minimal(sheet_body: &str, extra_parts: &[(&str, &str)], wb_extra_rels: &str, wb_extra: &str) -> Vec<u8>
