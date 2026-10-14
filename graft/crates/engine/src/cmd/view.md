# crates/engine/src/cmd/view.rs

- specs · function · L9-L108 — pub fn specs() -> Vec<CommandSpec>
- freeze_panes · function · L110-L123 — fn freeze_panes(s: &mut Session, p: &Json) -> Result<Json>
- set_freeze · function · L125-L131 — fn set_freeze(s: &mut Session, f: Option<(u32, u32)>) -> Result<Json>
- sheet_flag · function · L133-L148 — fn sheet_flag(s: &mut Session, p: &Json, which: &str) -> Result<Json>
- zoom · function · L150-L163 — fn zoom(s: &mut Session, p: &Json) -> Result<Json>
- zoom_selection · function · L165-L175 — fn zoom_selection(s: &mut Session, p: &Json) -> Result<Json>
- orientation · function · L177-L184 — fn orientation(s: &mut Session, p: &Json) -> Result<Json>
- paper · function · L186-L193 — fn paper(s: &mut Session, p: &Json) -> Result<Json>
- margins · function · L195-L217 — fn margins(s: &mut Session, p: &Json) -> Result<Json>
- print_area · function · L219-L227 — fn print_area(s: &mut Session, p: &Json) -> Result<Json>
- print_titles · function · L229-L239 — fn print_titles(s: &mut Session, p: &Json) -> Result<Json>
- scale_fit · function · L241-L254 — fn scale_fit(s: &mut Session, p: &Json) -> Result<Json>
- breaks · function · L256-L282 — fn breaks(s: &mut Session, p: &Json) -> Result<Json>
- header_footer · function · L284-L296 — fn header_footer(s: &mut Session, p: &Json) -> Result<Json>
- themes · function · L299-L309 — pub fn themes() -> Vec<(&'static str, [u32; 12])>
- theme · function · L311-L335 — fn theme(s: &mut Session, p: &Json) -> Result<Json>
- unused · function · L338-L338 — fn unused(_: CellRef) {}
