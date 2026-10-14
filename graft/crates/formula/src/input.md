# crates/formula/src/input.rs

- FormulaLocale · enum · L8-L12 — pub enum FormulaLocale
- SPANISH · constant · L14-L25 — const SPANISH: &[(&str, &str)] = &[
- list_separator · function · L28-L30 — pub fn list_separator(self) -> char
- decimal_separator · function · L31-L33 — pub fn decimal_separator(self) -> char
- array_column_separator · function · L34-L36 — pub fn array_column_separator(self) -> char
- function_name · function · L37-L43 — pub fn function_name(self, canonical: &str) -> &str
- canonical_name · function · L44-L50 — pub fn canonical_name(self, localized: &str) -> &str
- parse_input · function · L55-L61 — pub fn parse_input(src: &str, locale: FormulaLocale, known_name: impl Fn(&str) -> bool) -> Result<Expr, ParseError>
- print_input · function · L65-L115 — pub fn print_input(expr: &Expr, locale: FormulaLocale, known_name: impl Fn(&str) -> bool) -> Result<String, ParseError>
- boolean_alias · function · L117-L125 — pub(crate) fn boolean_alias(name: &str) -> Option<bool>
- normalize · function · L127-L204 — fn normalize(expr: &mut Expr, known_name: &impl Fn(&str) -> bool, bindings: &mut Vec<String>, depth: usize, display: bool) -> Result<(), ParseError>
