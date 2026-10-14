# crates/formula/src/printer.rs

- needs_quotes · function · L10-L20 — pub fn needs_quotes(name: &str) -> bool
- quote_sheet · function · L22-L24 — pub fn quote_sheet(name: &str) -> String
- sheet_prefix · function · L26-L35 — fn sheet_prefix(s: &SheetSel) -> String
- anchor_a1 · function · L37-L39 — fn anchor_a1(a: &Anchor) -> String
- reference_a1 · function · L41-L51 — pub fn reference_a1(r: &Reference) -> String
- r1c1_part · function · L53-L60 — fn r1c1_part(letter: char, v: u32, abs: bool, base: u32) -> String
- anchor_r1c1 · function · L62-L64 — fn anchor_r1c1(a: &Anchor, at: CellRef) -> String
- reference_r1c1 · function · L66-L80 — pub fn reference_r1c1(r: &Reference, at: CellRef) -> String
- struct_ref · function · L82-L121 — pub fn struct_ref(s: &StructRef) -> String
- number · function · L123-L125 — fn number(n: f64) -> String
- print · function · L128-L132 — pub fn print(e: &Expr) -> String
- print_r1c1 · function · L135-L139 — pub fn print_r1c1(e: &Expr, at: CellRef) -> String
- write_expr · function · L141-L231 — fn write_expr(s: &mut String, e: &Expr, r1c1: Option<CellRef>)
