# crates/xlsx/src/sheet_write.rs

- sqref · function · L19-L21 — fn sqref(ranges: &[RangeRef]) -> String
- cf_op_name · function · L23-L34 — fn cf_op_name(op: CfOperator) -> &'static str
- table_name · function · L37-L49 — fn table_name(name: &str, used: &mut HashSet<String>, n: u32) -> String
- table_columns · function · L52-L72 — fn table_columns(t: &Table) -> Vec<String>
- write_sheet · function · L74-L553 — pub fn write_sheet(wb: &Workbook, si: usize, selected: bool, out: &mut Out) -> (String, Rels)
- write_cell · function · L555-L653 — fn write_cell(
- cfvo_xml · function · L655-L664 — fn cfvo_xml(v: &CfValueKind) -> String
- quote · function · L666-L668 — fn quote(t: &str) -> String
- cf_rule_xml · function · L670-L796 — fn cf_rule_xml(cf: &CondFormat, priority: u32, out: &mut Out) -> String
- validation_formula · function · L798-L804 — fn validation_formula(f: &str) -> String
- validation_xml · function · L806-L852 — fn validation_xml(v: &Validation) -> String
- filter_operator · function · L854-L863 — fn filter_operator(op: &str) -> &'static str
- autofilter_xml · function · L865-L910 — fn autofilter_xml(af: &AutoFilter, out: &mut Out) -> String
- table_xml · function · L912-L980 — fn table_xml(t: &Table, cols: &[String], name: &str, id: u32) -> String
- comments_xml · function · L982-L1005 — fn comments_xml(sheet: &Sheet) -> String
- vml_xml · function · L1008-L1030 — fn vml_xml(sheet: &Sheet, n: u32) -> String
