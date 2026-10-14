# crates/ui-egui/src/icons.rs

- Icon · enum · L8-L115 — pub enum Icon
- G · struct · L117-L121 — struct G<'a>
- pt · function · L124-L126 — fn pt(&self, x: f32, y: f32) -> Pos2
- line · function · L127-L130 — fn line(&self, pts: &[(f32, f32)], c: Color32, w: f32)
- stroke · function · L131-L133 — fn stroke(&self, pts: &[(f32, f32)], c: Color32)
- rect · function · L134-L142 — fn rect(&self, x: f32, y: f32, w: f32, h: f32, fill: Option<Color32>, stroke: Option<Color32>)
- poly · function · L143-L146 — fn poly(&self, pts: &[(f32, f32)], fill: Color32)
- circle · function · L147-L154 — fn circle(&self, x: f32, y: f32, r: f32, fill: Option<Color32>, stroke: Option<Color32>)
- text · function · L155-L165 — fn text(&self, x: f32, y: f32, t: &str, size: f32, c: Color32, bold: bool, italic: bool)
- GREEN · constant · L169-L169 — pub const GREEN: Color32 = Color32::from_rgb(0x10, 0x7C, 0x41);
- BLUE · constant · L170-L170 — pub const BLUE: Color32 = Color32::from_rgb(0x2B, 0x7C, 0xD3);
- RED · constant · L171-L171 — pub const RED: Color32 = Color32::from_rgb(0xD1, 0x3B, 0x2F);
- YELLOW · constant · L172-L172 — pub const YELLOW: Color32 = Color32::from_rgb(0xF2, 0xC3, 0x2E);
- ORANGE · constant · L173-L173 — pub const ORANGE: Color32 = Color32::from_rgb(0xE8, 0x7D, 0x2B);
- PURPLE · constant · L174-L174 — pub const PURPLE: Color32 = Color32::from_rgb(0x88, 0x55, 0xC8);
- paint · function · L177-L615 — pub fn paint(p: &Painter, rect: Rect, icon: Icon, c: Color32)
- LIBRARY · constant · L618-L649 — pub const LIBRARY: &[(&str, Icon)] = &[
- from_name · function · L651-L653 — pub fn from_name(name: &str) -> Option<Icon>
