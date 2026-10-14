# crates/ui-egui/src/assistant_pane.rs

- Runtime · enum · L19-L24 — enum Runtime
- Line · enum · L27-L32 — enum Line
- Active · struct · L34-L38 — struct Active
- AssistantPane · struct · L40-L50 — pub struct AssistantPane
- default · function · L53-L65 — fn default() -> Self
- start_runtime · function · L69-L84 — fn start_runtime(&mut self)
- is_busy · function · L86-L88 — pub fn is_busy(&self) -> bool
- ask · function · L92-L104 — pub fn ask(app: &mut SheetApp, prompt: &str) -> Result<Json, String>
- begin · function · L106-L117 — fn begin(app: &mut SheetApp, prompt: String)
- tick · function · L120-L173 — fn tick(app: &mut SheetApp, ctx: &egui::Context)
- live_status · function · L175-L185 — fn live_status(live: &str) -> String
- show · function · L187-L291 — pub fn show(app: &mut SheetApp, ui: &mut Ui)
