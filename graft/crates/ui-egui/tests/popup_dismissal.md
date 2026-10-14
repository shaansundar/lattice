# crates/ui-egui/tests/popup_dismissal.rs

- Harness · type · L10-L10 — type Harness = egui_kittest::Harness<'static, SheetApp>;
- Menu · enum · L13-L17 — enum Menu
- harness · function · L19-L39 — fn harness() -> Harness
- geo · function · L41-L45 — fn geo(h: &Harness) -> grid::Geo
- cell_pos · function · L47-L49 — fn cell_pos(h: &Harness, cell: &str) -> Pos2
- click · function · L51-L56 — fn click(h: &mut Harness, pos: Pos2, button: PointerButton)
- open · function · L58-L74 — fn open(h: &mut Harness, menu: Menu)
- is_open · function · L76-L82 — fn is_open(h: &Harness, menu: Menu) -> bool
- worksheet_popups_close_on_grid_and_ribbon_clicks · function · L85-L98 — fn worksheet_popups_close_on_grid_and_ribbon_clicks()
- filter_closes_when_clicking_another_table_or_sheet · function · L101-L114 — fn filter_closes_when_clicking_another_table_or_sheet()
- popup_actions_still_execute_and_filter_checkboxes_stay_open · function · L117-L152 — fn popup_actions_still_execute_and_filter_checkboxes_stay_open()
- worksheet_popups_close_on_escape_or_window_focus_loss · function · L155-L176 — fn worksheet_popups_close_on_escape_or_window_focus_loss()
