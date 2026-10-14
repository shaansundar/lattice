# crates/engine/tests/cross_sheet_cut.rs

- workbook · function · L5-L9 — fn workbook() -> Session
- run · function · L11-L13 — fn run(s: &mut Session, command: &str, params: Json)
- set · function · L15-L17 — fn set(s: &mut Session, cell: &str, input: &str)
- activate · function · L19-L21 — fn activate(s: &mut Session, sheet: &str)
- value · function · L23-L26 — fn value(s: &Session, sheet: &str, cell: &str) -> Value
- formula · function · L28-L31 — fn formula(s: &Session, sheet: &str, cell: &str) -> String
- cut_to · function · L33-L38 — fn cut_to(s: &mut Session, source: &str, range: &str, destination: &str, at: &str)
- cross_sheet_cut_preserves_total_through_undo_redo_and_xlsx · function · L41-L70 — fn cross_sheet_cut_preserves_total_through_undo_redo_and_xlsx()
- references_on_all_sheets_follow_cut_but_destination_local_refs_do_not · function · L73-L102 — fn references_on_all_sheets_follow_cut_but_destination_local_refs_do_not()
- moved_formulas_keep_external_source_refs_and_move_internal_refs_once · function · L105-L129 — fn moved_formulas_keep_external_source_refs_and_move_internal_refs_once()
- same_sheet_overlapping_cut_still_updates_references_once · function · L132-L146 — fn same_sheet_overlapping_cut_still_updates_references_once()
