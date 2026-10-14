# crates/numfmt/src/render.rs

- Out · struct · L11-L14 — pub(crate) struct Out
- push · function · L17-L19 — fn push(&mut self, s: &str)
- fill · function · L20-L24 — fn fill(&mut self, c: char)
- prepend · function · L25-L30 — pub(crate) fn prepend(&mut self, s: &str)
- render_text · function · L34-L44 — pub(crate) fn render_text(sec: &Section, text: &str) -> Out
- render_general · function · L47-L57 — pub(crate) fn render_general(sec: &Section, v: f64, width: usize) -> Out
- render_date · function · L60-L138 — pub(crate) fn render_date(sec: &Section, v: f64, sys: DateSystem) -> Option<Out>
- fill_int · function · L142-L179 — fn fill_int(toks: &[Tok], idxs: &[usize], digits: &str, thousands: bool, force_zero: bool, pieces: &mut [String])
- with_commas · function · L181-L194 — fn with_commas(digits: &str, thousands: bool) -> String
- fill_frac · function · L197-L218 — fn fill_frac(toks: &[Tok], idxs: &[usize], frac: &str, pieces: &mut [String])
- digit_indices · function · L220-L222 — fn digit_indices(toks: &[Tok], range: std::ops::Range<usize>) -> Vec<usize>
- render_number · function · L225-L332 — pub(crate) fn render_number(sec: &Section, v: f64) -> Out
- approximate · function · L335-L371 — fn approximate(x: f64, max_den: u64) -> (u64, u64)
- render_fraction · function · L373-L464 — fn render_fraction(sec: &Section, x: f64, pieces: &mut [String])
- tests · module · L467-L479 — mod tests
- approximations · function · L471-L478 — fn approximations()
