# crates/engine/src/cmd/draw.rs

- specs · function · L9-L28 — pub fn specs() -> Vec<CommandSpec>
- pen · function · L30-L40 — fn pen(s: &mut Session, p: &Json) -> Result<Json>
- eraser · function · L42-L46 — fn eraser(s: &mut Session, p: &Json) -> Result<Json>
- parse_points · function · L48-L50 — fn parse_points(t: &str) -> Vec<[f32; 2]>
- stroke · function · L52-L102 — fn stroke(s: &mut Session, p: &Json) -> Result<Json>
- ink_width · function · L105-L110 — pub fn ink_width(sh: &Shape) -> f32
- ink_points · function · L112-L114 — pub fn ink_points(sh: &Shape) -> Vec<[f32; 2]>
- ink_to_shape · function · L116-L157 — fn ink_to_shape(s: &mut Session, p: &Json) -> Result<Json>
