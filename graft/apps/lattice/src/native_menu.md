# apps/lattice/src/native_menu.rs

- Entry · type · L12-L12 — type Entry = (&'static str, &'static str, Option<&'static str>);
- tree · function · L15-L178 — fn tree() -> Vec<(&'static str, Vec<Entry>)>
- NativeMenu · struct · L180-L184 — pub struct NativeMenu
- select_theme · function · L186-L195 — fn select_theme(app: &mut SheetApp, id: &str) -> bool
- install · function · L198-L246 — pub fn install(ctx: &egui::Context) -> NativeMenu
- poll · function · L248-L307 — pub fn poll(&self, app: &mut SheetApp, ctx: &egui::Context)
- tests · module · L311-L325 — mod tests
- theme_choices_use_the_shared_preference_command · function · L315-L324 — fn theme_choices_use_the_shared_preference_command()
