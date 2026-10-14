# crates/xlsx/src/array_formula.rs

- Shape · struct · L12-L15 — struct Shape
- array · function · L18-L20 — fn array() -> Self
- needs_array · function · L23-L31 — pub(crate) fn needs_array(wb: &Workbook, sheet: usize, at: CellRef, formula: &Formula) -> bool
- Scan · struct · L33-L38 — struct Scan<'a>
- expr · function · L41-L101 — fn expr(&mut self, e: &Expr, sheet: usize, array_context: bool, depth: usize) -> Shape
- call · function · L103-L182 — fn call(&mut self, name: &str, args: &[Expr], sheet: usize, array_context: bool, depth: usize) -> Shape
- positive · function · L185-L187 — fn positive(e: &Expr) -> bool
- one · function · L189-L191 — fn one(e: &Expr) -> bool
- single_column · function · L193-L195 — fn single_column(e: &Expr) -> bool
- scalar_arg · function · L199-L223 — fn scalar_arg(name: &str, i: usize) -> bool
