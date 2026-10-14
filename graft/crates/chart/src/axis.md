# crates/chart/src/axis.rs

- MAX_ABS · constant · L7-L7 — pub(crate) const MAX_ABS: f64 = 1e300;
- Scale · struct · L11-L15 — pub struct Scale
- ticks · function · L19-L33 — pub fn ticks(&self) -> Vec<f64>
- frac · function · L35-L41 — pub fn frac(&self, v: f64) -> f32
- clean · function · L44-L46 — pub(crate) fn clean(v: f64) -> f64
- nice_scale · function · L53-L108 — pub fn nice_scale(lo: f64, hi: f64, force_zero: bool) -> Scale
- fixed_scale · function · L111-L113 — pub(crate) fn fixed_scale(min: f64, max: f64, step: f64) -> Scale
- Fmt · struct · L116-L119 — pub(crate) struct Fmt
- new · function · L122-L126 — pub fn new(code: &str) -> Fmt
- format · function · L128-L144 — pub fn format(&self, v: f64, step: Option<f64>) -> String
- general_text · function · L147-L159 — fn general_text(v: f64, step: Option<f64>) -> String
- format_number · function · L162-L164 — pub fn format_number(v: f64, code: &str, step: Option<f64>) -> String
