# crates/xlsx/src/tests/array_export.rs

- at · function · L12-L14 — fn at(s: &str) -> CellRef
- range · function · L16-L18 — fn range(s: &str) -> RangeRef
- formula · function · L20-L24 — fn formula(text: &str, value: f64) -> Cell
- cell · function · L26-L28 — fn cell<'a>(sheet: &'a El, address: &str) -> &'a El
- assert_scalar · function · L30-L36 — fn assert_scalar(sheet: &El, address: &str)
- assert_dynamic · function · L38-L44 — fn assert_dynamic(sheet: &El, address: &str, reference: &str)
- nonspilling_formulas_preserve_array_evaluation_and_metadata · function · L47-L97 — fn nonspilling_formulas_preserve_array_evaluation_and_metadata()
- dynamic_spills_and_legacy_cse_keep_distinct_ranges · function · L100-L133 — fn dynamic_spills_and_legacy_cse_keep_distinct_ranges()
- scalar_formulas_constants_and_legacy_cse_do_not_require_dynamic_metadata · function · L136-L162 — fn scalar_formulas_constants_and_legacy_cse_do_not_require_dynamic_metadata()
- calculated_table_column_keeps_formula_and_cell_value · function · L165-L200 — fn calculated_table_column_keeps_formula_and_cell_value()
- check_formula_metadata · function · L203-L224 — fn check_formula_metadata(mut wb: Workbook, sheet_index: usize, cases: &[(&str, bool)])
- implicit_intersection_metadata_respects_function_argument_context · function · L227-L264 — fn implicit_intersection_metadata_respects_function_argument_context()
- implicit_intersection_metadata_resolves_names_in_formula_sheet_scope · function · L267-L296 — fn implicit_intersection_metadata_resolves_names_in_formula_sheet_scope()
- cyclic_names_preserve_array_evaluation_without_unbounded_export_recursion · function · L299-L305 — fn cyclic_names_preserve_array_evaluation_without_unbounded_export_recursion()
- implicit_intersection_metadata_distinguishes_table_rows_and_columns · function · L308-L339 — fn implicit_intersection_metadata_distinguishes_table_rows_and_columns()
