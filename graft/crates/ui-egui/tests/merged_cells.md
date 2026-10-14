# crates/ui-egui/tests/merged_cells.rs

- Harness · type · L10-L10 — type Harness = egui_kittest::Harness<'static, SheetApp>;
- harness · function · L12-L26 — fn harness(input: &str) -> Harness
- cell_center · function · L28-L33 — fn cell_center(h: &Harness, cell: &str) -> Pos2
- click_cell · function · L35-L43 — fn click_cell(h: &mut Harness, cell: &str)
- text · function · L45-L48 — fn text(h: &mut Harness, text: &str)
- key · function · L50-L55 — fn key(h: &mut Harness, key: Key)
- value · function · L57-L59 — fn value(h: &Harness, cell: &str) -> Value
- typing_into_a_covered_cell_remains_visible_after_clicking_away · function · L62-L74 — fn typing_into_a_covered_cell_remains_visible_after_clicking_away()
- f2_and_double_click_edit_the_existing_merged_content · function · L77-L98 — fn f2_and_double_click_edit_the_existing_merged_content()
- formula_bar_reads_and_edits_the_merge_anchor_from_a_covered_cell · function · L101-L118 — fn formula_bar_reads_and_edits_the_merge_anchor_from_a_covered_cell()
