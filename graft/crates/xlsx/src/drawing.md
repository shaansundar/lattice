# crates/xlsx/src/drawing.rs

- emu_px · function · L16-L18 — fn emu_px(v: Option<i64>) -> f32
- marker · function · L20-L25 — fn marker(e: &El) -> (CellRef, f32, f32)
- ext_px · function · L27-L32 — fn ext_px(e: Option<&El>) -> (f32, f32)
- read_anchor · function · L34-L75 — fn read_anchor(a: &El, sheet: &Sheet) -> Option<Anchor>
- dml_color · function · L78-L104 — pub fn dml_color(e: &El) -> Option<Color>
- mime_for · function · L106-L120 — fn mime_for(name: &str) -> &'static str
- read_drawing · function · L122-L194 — pub fn read_drawing(cx: &mut Ctx<'_>, part: &str, sheet: &mut Sheet) -> Result<(), IoError>
- read_shape · function · L196-L221 — fn read_shape(obj: &El, anchor: Anchor) -> Option<Shape>
- anchor_open · function · L230-L255 — fn anchor_open(sheet: &Sheet, a: &Anchor) -> (String, &'static str)
- xfrm · function · L257-L260 — fn xfrm(a: &Anchor, tag: &str) -> String
- srgb · function · L262-L265 — pub fn srgb(c: &Color, theme: &Theme) -> Option<String>
- Obj · enum · L268-L272 — pub enum Obj<'a>
- write_drawing · function · L275-L377 — pub fn write_drawing(sheet: &Sheet, theme: &Theme, objs: &[Obj<'_>]) -> String
- image_ext · function · L380-L403 — pub fn image_ext(mime: &str, data: &[u8]) -> &'static str
- image_content_type · function · L405-L417 — pub fn image_content_type(ext: &str) -> &'static str
