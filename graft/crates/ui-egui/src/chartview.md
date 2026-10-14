# crates/ui-egui/src/chartview.rs

- rgba · function · L16-L18 — fn rgba(c: [u8; 4]) -> Color32
- EguiMeasure · struct · L20-L23 — struct EguiMeasure<'a>
- text_width · function · L26-L29 — fn text_width(&self, text: &str, size: f32, bold: bool) -> f32
- paint_prims · function · L33-L138 — pub fn paint_prims(p: &Painter, origin: Pos2, prims: &[Prim], z: f32)
- is_convex · function · L140-L157 — fn is_convex(v: &[Pos2]) -> bool
- area_mesh · function · L161-L181 — fn area_mesh(v: &[Pos2], col: Color32) -> egui::Mesh
- anchor_rect · function · L183-L187 — pub(crate) fn anchor_rect(geo: &Geo, sh: &Sheet, a: &Anchor) -> Rect
- TEXTURES · constant · L189-L189 — static TEXTURES: std::sync::LazyLock<Mutex<HashMap<(u32, usize), egui::TextureHandle>>> = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
- paint_objects · function · L191-L311 — pub fn paint_objects(app: &SheetApp, p: &Painter, geo: &Geo, wb: &Workbook, si: usize, sh: &Sheet)
- selection_frame · function · L313-L320 — fn selection_frame(p: &Painter, r: Rect)
- hit · function · L323-L344 — pub fn hit(app: &SheetApp, geo: &Geo, sh: &Sheet, p: Pos2) -> Option<(&'static str, u32, Rect)>
