# crates/ui-egui/src/formula_locale.rs

- canonical · function · L5-L40 — pub(crate) fn canonical(
- display · function · L42-L53 — pub(crate) fn display(text: &str, locale: FormulaLocale, wb: &Workbook, sheet: usize) -> String
- SPANISH_HELP · constant · L57-L68 — const SPANISH_HELP: &[(&str, &str, &str)] = &[
- signature · function · L70-L78 — pub(crate) fn signature(name: &str, locale: FormulaLocale) -> Option<String>
- description · function · L80-L88 — pub(crate) fn description(name: &str, locale: FormulaLocale) -> Option<String>
- completion_name · function · L91-L94 — pub(crate) fn completion_name<'a>(name: &'a str, locale: FormulaLocale, wb: &Workbook, sheet: usize) -> &'a str
- display_template · function · L98-L106 — pub(crate) fn display_template(text: &str, locale: FormulaLocale, wb: &Workbook, sheet: usize) -> String
