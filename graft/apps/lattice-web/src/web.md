# apps/lattice-web/src/web.rs

- CANVAS_ID · constant · L7-L7 — const CANVAS_ID: &str = "lattice_canvas";
- LOADING_ID · constant · L8-L8 — const LOADING_ID: &str = "lattice_loading";
- start · function · L10-L62 — pub fn start()
- query · function · L64-L66 — fn query() -> String
- WebShell · struct · L68-L71 — struct WebShell
- logic · function · L74-L91 — fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame)
- raw_input_hook · function · L93-L95 — fn raw_input_hook(&mut self, _ctx: &egui::Context, raw: &mut egui::RawInput)
- ui · function · L97-L99 — fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame)
- services · function · L102-L147 — fn services(inbox: Inbox, ctx: egui::Context) -> Services
- copy_html · function · L150-L180 — fn copy_html(html: &str, text: &str) -> Result<(), String>
- download · function · L183-L205 — fn download(name: &str, bytes: &[u8]) -> Result<(), String>
