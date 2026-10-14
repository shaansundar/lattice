# crates/engine/src/selection.rs

- Selection · struct · L8-L15 — pub struct Selection
- default · function · L18-L20 — fn default() -> Self
- at · function · L24-L26 — pub fn at(c: CellRef) -> Selection
- range · function · L27-L29 — pub fn range(r: RangeRef) -> Selection
- current · function · L31-L33 — pub fn current(&self) -> RangeRef
- bounds · function · L35-L39 — pub fn bounds(&self) -> RangeRef
- contains · function · L40-L42 — pub fn contains(&self, c: CellRef) -> bool
- is_single_cell · function · L43-L45 — pub fn is_single_cell(&self) -> bool
- extend_to · function · L47-L54 — pub fn extend_to(&mut self, to: CellRef)
- a1 · function · L56-L58 — pub fn a1(&self) -> String
- expand_merges · function · L60-L77 — pub fn expand_merges(&mut self, sheet: &Sheet)
- cycle · function · L79-L117 — pub fn cycle(&mut self, forward: bool, by_row: bool)
- jump · function · L121-L178 — pub fn jump(sheet: &Sheet, from: CellRef, dr: i64, dc: i64) -> CellRef
- current_region · function · L181-L210 — pub fn current_region(sheet: &Sheet, c: CellRef) -> RangeRef
- tests · module · L213-L263 — mod tests
- jumps · function · L218-L231 — fn jumps()
- regions · function · L234-L251 — fn regions()
- cycling · function · L254-L262 — fn cycling()
