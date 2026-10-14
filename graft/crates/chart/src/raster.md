# crates/chart/src/raster.rs

- MAX_PIXELS · constant · L8-L8 — const MAX_PIXELS: u64 = 64 * 1024 * 1024;
- SUB · constant · L9-L9 — const SUB: usize = 4;
- Path · type · L11-L11 — type Path = Vec<Vec<[f32; 2]>>;
- Canvas · struct · L13-L18 — struct Canvas
- blend · function · L21-L40 — fn blend(&mut self, x: usize, y: usize, c: Rgba, cov: f32)
- fill · function · L43-L120 — fn fill(&mut self, path: &Path, color: Rgba)
- arc · function · L123-L132 — fn arc(cx: f32, cy: f32, r: f32, a0: f32, a1: f32) -> Vec<[f32; 2]>
- circle · function · L134-L138 — fn circle(cx: f32, cy: f32, r: f32) -> Vec<[f32; 2]>
- rounded_rect · function · L140-L152 — fn rounded_rect(x: f32, y: f32, w: f32, h: f32, r: f32) -> Vec<[f32; 2]>
- stroke_paths · function · L155-L193 — fn stroke_paths(pts: &[[f32; 2]], width: f32, dash: bool, closed: bool) -> Vec<Path>
- rasterize · function · L197-L263 — pub fn rasterize(prims: &[Prim], w: u32, h: u32, background: Rgba) -> Vec<u8>
