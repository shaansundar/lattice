# apps/lattice/src/main.rs

- apple_events · module · L12-L12 — mod apple_events;
- control_server · module · L13-L13 — mod control_server;
- graphics · module · L15-L15 — mod graphics;
- logging · module · L16-L16 — mod logging;
- native_menu · module · L18-L18 — mod native_menu;
- App · struct · L24-L24 — struct App(SheetApp, #[cfg(target_os = "macos")] Option<native_menu::NativeMenu>, #[cfg(target_os = "macos")] fmv_macos_events::Inbox);
- logic · function · L27-L39 — fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
- raw_input_hook · function · L40-L48 — fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput)
- ui · function · L49-L51 — fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame)
- on_exit · function · L52-L54 — fn on_exit(&mut self)
- config_dir · function · L57-L68 — fn config_dir() -> Option<std::path::PathBuf>
- prefs_enabled · function · L72-L74 — fn prefs_enabled() -> bool
- log_dir · function · L77-L79 — fn log_dir() -> Option<std::path::PathBuf>
- load_prefs · function · L81-L101 — fn load_prefs(app: &mut SheetApp)
- save_prefs · function · L103-L115 — fn save_prefs(app: &SheetApp)
- services · function · L117-L165 — fn services() -> Services
- icon · function · L167-L174 — fn icon() -> Option<egui::IconData>
- main · function · L176-L299 — fn main() -> eframe::Result<()>
