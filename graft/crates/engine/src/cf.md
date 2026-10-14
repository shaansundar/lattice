# crates/engine/src/cf.rs

- CfLook · struct · L11-L21 — pub struct CfLook
- CfCache · struct · L25-L27 — pub struct CfCache
- RuleStats · struct · L30-L35 — struct RuleStats
- numbers_in · function · L37-L57 — fn numbers_in(sh: &Sheet, ranges: &[RangeRef]) -> (Vec<f64>, HashMap<String, usize>)
- percentile · function · L59-L69 — fn percentile(sorted: &[f64], p: f64) -> f64
- threshold · function · L71-L82 — fn threshold(wb: &Workbook, sheet: usize, at: CellRef, st: &RuleStats, k: &CfValueKind) -> f64
- lerp · function · L84-L91 — fn lerp(a: [u8; 3], b: [u8; 3], t: f64) -> [u8; 3]
- new · function · L96-L98 — pub fn new() -> Self
- look · function · L101-L298 — pub fn look(&mut self, wb: &Workbook, sheet: usize, c: CellRef) -> Option<CfLook>
- relative · function · L302-L312 — fn relative(f: &str, first: CellRef, c: CellRef) -> String
