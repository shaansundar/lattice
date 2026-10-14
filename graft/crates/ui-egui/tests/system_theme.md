# crates/ui-egui/tests/system_theme.rs

- harness · function · L9-L24 — fn harness(ui: UiState) -> egui_kittest::Harness<'static, SheetApp>
- preferences · function · L26-L28 — fn preferences(h: &egui_kittest::Harness<'static, SheetApp>) -> Value
- system_theme · function · L30-L33 — fn system_theme(h: &mut egui_kittest::Harness<'static, SheetApp>, theme: Option<Theme>)
- set_mode · function · L35-L44 — fn set_mode(h: &mut egui_kittest::Harness<'static, SheetApp>, mode: &str)
- select_ribbon_theme · function · L46-L52 — fn select_ribbon_theme(h: &mut egui_kittest::Harness<'static, SheetApp>, label: &str)
- assert_appearance · function · L54-L61 — fn assert_appearance(h: &egui_kittest::Harness<'static, SheetApp>, dark: bool)
- old_preferences_stay_manual_and_system_preference_survives_reload · function · L64-L84 — fn old_preferences_stay_manual_and_system_preference_survives_reload()
- system_tracks_live_os_changes_with_both_lattice_palettes_and_light_fallback · function · L87-L104 — fn system_tracks_live_os_changes_with_both_lattice_palettes_and_light_fallback()
- ribbon_selects_system_and_fixed_modes_ignore_subsequent_os_changes · function · L107-L126 — fn ribbon_selects_system_and_fixed_modes_ignore_subsequent_os_changes()
- invalid_mode_preserves_preferences_and_legacy_overrides_leave_system_mode · function · L129-L164 — fn invalid_mode_preserves_preferences_and_legacy_overrides_leave_system_mode()
