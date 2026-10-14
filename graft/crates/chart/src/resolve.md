# crates/chart/src/resolve.rs

- MAX_POINTS · constant · L11-L11 — pub(crate) const MAX_POINTS: usize = 100_000;
- MAX_SERIES · constant · L13-L13 — pub(crate) const MAX_SERIES: usize = 255;
- CYCLE_TINTS · constant · L16-L16 — const CYCLE_TINTS: [f64; 6] = [0.0, 0.4, -0.25, 0.6, -0.5, 0.2];
- series_color · function · L19-L27 — pub fn series_color(theme: &Theme, i: usize) -> Rgba
- default_palette · function · L30-L32 — pub fn default_palette(theme: &Theme, n: usize) -> Vec<Rgba>
- parse_ref · function · L35-L47 — fn parse_ref(wb: &Workbook, sheet: usize, formula: &str) -> Option<(usize, RangeRef)>
- eval_list · function · L50-L59 — fn eval_list(wb: &Workbook, sheet: usize, formula: &str) -> Vec<Value>
- to_num · function · L61-L66 — fn to_num(v: &Value) -> Option<f64>
- nums · function · L68-L70 — fn nums(wb: &Workbook, sheet: usize, formula: &str) -> Vec<Option<f64>>
- ref_format · function · L73-L77 — fn ref_format(wb: &Workbook, sheet: usize, formula: &str) -> String
- value_text · function · L79-L88 — fn value_text(v: &Value) -> String
- category_texts · function · L91-L111 — fn category_texts(wb: &Workbook, sheet: usize, formula: &str) -> Vec<String>
- series_name · function · L114-L125 — fn series_name(wb: &Workbook, sheet: usize, name: &str) -> String
- effective_kind · function · L127-L140 — fn effective_kind(chart_kind: ChartKind, series_kind: Option<ChartKind>, i: usize) -> ChartKind
- resolve · function · L145-L184 — pub fn resolve(wb: &Workbook, sheet: usize, chart: &Chart) -> ChartData
