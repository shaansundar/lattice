# crates/engine/src/display.rs

- parsed · function · L11-L23 — fn parsed(code: &str) -> NumberFormat
- CACHE · constant · L12-L12 — static CACHE: std::sync::LazyLock<Mutex<HashMap<String, NumberFormat>>> = std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));
- number_format · function · L25-L27 — pub fn number_format(code: &str) -> NumberFormat
- format · function · L30-L32 — pub fn format(v: &Value, code: &str, wb: &Workbook) -> Formatted
- formula_hidden · function · L36-L48 — pub fn formula_hidden(wb: &Workbook, sheet: &Sheet, c: CellRef) -> bool
- cell_text · function · L51-L71 — pub fn cell_text(wb: &Workbook, sheet: &Sheet, c: CellRef) -> String
- range_text · function · L74-L100 — pub fn range_text(wb: &Workbook, sheet: usize, r: RangeRef) -> String
- clipboard_range · function · L103-L105 — pub(crate) fn clipboard_range(r: RangeRef) -> RangeRef
- Stats · struct · L110-L117 — pub struct Stats
- stats · function · L119-L158 — pub fn stats(sheet: &Sheet, ranges: &[RangeRef]) -> Stats
