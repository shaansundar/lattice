# crates/ui-egui/tests/selection.rs

- Harness · type · L9-L9 — type Harness = egui_kittest::Harness<'static, SheetApp>;
- harness · function · L11-L28 — fn harness(initial: &str) -> Harness
- point · function · L30-L34 — fn point(h: &Harness, a1: &str) -> Pos2
- cells · function · L36-L39 — fn cells(h: &Harness) -> Value
- frame · function · L41-L44 — fn frame(h: &mut Harness, event: Event)
- button · function · L46-L48 — fn button(pos: Pos2, pressed: bool) -> Event
- press · function · L50-L54 — fn press(h: &mut Harness, cell: &str)
- assert_selection · function · L56-L62 — fn assert_selection(h: &Harness, start: &str, end: &str)
- single_movement_selects_from_the_pressed_cell_in_both_directions · function · L65-L78 — fn single_movement_selects_from_the_pressed_cell_in_both_directions()
- release_consumes_a_new_endpoint_without_an_intermediate_movement_frame · function · L81-L91 — fn release_consumes_a_new_endpoint_without_an_intermediate_movement_frame()
- multiple_movement_frames_keep_the_anchor_and_cell_contents · function · L94-L107 — fn multiple_movement_frames_keep_the_anchor_and_cell_contents()
- clicking_then_hovering_does_not_extend_selection · function · L110-L120 — fn clicking_then_hovering_does_not_extend_selection()
- formula_point_mode_consumes_first_movement_and_release · function · L123-L139 — fn formula_point_mode_consumes_first_movement_and_release()
- formula_point_drag_keeps_the_sheet_qualifier · function · L142-L161 — fn formula_point_drag_keeps_the_sheet_qualifier()
