# crates/model/src/cell.rs

- Formula · struct · L13-L20 — pub struct Formula
- new · function · L25-L31 — pub fn new(text: &str) -> Formula
- from_expr · function · L32-L34 — pub fn from_expr(e: Expr) -> Formula
- expr · function · L36-L38 — pub fn expr(&self) -> Option<Expr>
- expr_arc · function · L40-L42 — pub fn expr_arc(&self) -> Option<Arc<Expr>>
- CellPicture · struct · L47-L53 — pub struct CellPicture
- Cell · struct · L56-L61 — pub struct Cell
- value · function · L64-L66 — pub fn value(v: Value) -> Cell
- formula · function · L67-L69 — pub fn formula(f: Formula) -> Cell
- is_blank · function · L71-L73 — pub fn is_blank(&self) -> bool
- input_text · function · L75-L87 — pub fn input_text(&self) -> String
- bar_text · function · L90-L95 — pub fn bar_text(&self) -> String
