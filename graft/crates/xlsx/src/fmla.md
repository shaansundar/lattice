# crates/xlsx/src/fmla.rs

- to_file · function · L8-L13 — pub fn to_file(f: &Formula) -> String
- text_to_file · function · L16-L22 — pub fn text_to_file(text: &str) -> String
- PARAM_PREFIX · constant · L24-L24 — const PARAM_PREFIX: &str = "_xlpm.";
- prefix_params · function · L27-L70 — fn prefix_params(e: Expr, scope: &[String]) -> Expr
- expr_to_file · function · L72-L86 — fn expr_to_file(e: Expr) -> String
- strip_param_prefix · function · L89-L94 — fn strip_param_prefix(n: String) -> String
- from_file · function · L97-L107 — pub fn from_file(text: &str) -> String
- tests · module · L110-L150 — mod tests
- spill_references · function · L114-L118 — fn spill_references()
- implicit_intersection · function · L121-L127 — fn implicit_intersection()
- prefixes · function · L130-L135 — fn prefixes()
- let_lambda_params · function · L138-L149 — fn let_lambda_params()
