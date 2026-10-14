# crates/ui-egui/src/formula_bar.rs

- show · function · L11-L63 — pub fn show(app: &mut SheetApp, ui: &mut egui::Ui)
- active_input · function · L65-L96 — fn active_input(app: &SheetApp) -> String
- name_box · function · L98-L155 — fn name_box(app: &mut SheetApp, ui: &mut egui::Ui, t: &Tokens)
- editor_widget · function · L158-L400 — pub fn editor_widget(app: &mut SheetApp, ui: &mut egui::Ui, id: egui::Id, font: FontId, multiline: bool, width: f32)
- cycle_anchor · function · L402-L435 — fn cycle_anchor(ed: &mut crate::editor::EditState)
- function_names · function · L437-L442 — fn function_names() -> Vec<String>
- NAMES · constant · L438-L438 — static NAMES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
- function_info · function · L444-L450 — fn function_info(name: &str) -> Option<serde_json::Value>
- LIST · constant · L445-L445 — static LIST: std::sync::OnceLock<Vec<serde_json::Value>> = std::sync::OnceLock::new();
- function_signature · function · L452-L454 — pub fn function_signature(name: &str) -> Option<String>
- function_description · function · L456-L458 — pub fn function_description(name: &str) -> Option<String>
