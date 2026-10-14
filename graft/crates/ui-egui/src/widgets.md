# crates/ui-egui/src/widgets.rs

- tr · function · L13-L15 — fn tr<'a>(ui: &Ui, text: &'a str) -> &'a str
- icon_button · function · L18-L28 — pub fn icon_button(ui: &mut Ui, icon: Icon, color: Color32, tip: &str, size: egui::Vec2) -> Response
- toggle_button · function · L31-L42 — pub fn toggle_button(ui: &mut Ui, icon: Icon, on: bool, tip: &str) -> Response
- big_button · function · L45-L77 — pub fn big_button(ui: &mut Ui, icon: Icon, label: &str, tip: &str, dropdown: bool) -> Response
- small_button · function · L80-L105 — pub fn small_button(ui: &mut Ui, icon: Icon, label: &str, tip: &str, dropdown: bool) -> Response
- split_button · function · L109-L126 — pub fn split_button(ui: &mut Ui, icon: Icon, accent: Option<Color32>, tip: &str) -> (bool, Response)
- color_palette · function · L130-L174 — pub fn color_palette(ui: &mut Ui, theme_colors: &[u32; 12], none_label: &str) -> Option<String>
- message_box · function · L176-L200 — pub fn message_box(app: &mut SheetApp, ctx: &egui::Context)
- toast · function · L202-L219 — pub fn toast(app: &mut SheetApp, ctx: &egui::Context)
