# crates/ui-egui/src/text_box.rs

- EditState · struct · L10-L18 — pub struct EditState
- editor_id · function · L20-L22 — pub(crate) fn editor_id() -> egui::Id
- begin_text_box_edit · function · L25-L57 — pub fn begin_text_box_edit(&mut self, id: u32) -> bool
- commit_text_box_edit · function · L59-L80 — pub fn commit_text_box_edit(&mut self) -> Result<(), String>
- cancel_text_box_edit · function · L82-L85 — pub fn cancel_text_box_edit(&mut self)
- editing_text_box · function · L87-L89 — pub(crate) fn editing_text_box(&self, id: u32) -> bool
- before_ui · function · L93-L107 — pub(crate) fn before_ui(app: &mut SheetApp, ctx: &egui::Context)
- contains_pointer · function · L109-L111 — pub(crate) fn contains_pointer(app: &SheetApp, ctx: &egui::Context) -> bool
- show · function · L113-L173 — pub(crate) fn show(app: &mut SheetApp, ui: &mut egui::Ui, geo: &Geo, sh: &lattice_engine::model::Sheet)
