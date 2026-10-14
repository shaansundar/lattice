# crates/numfmt/src/decimal.rs

- MAX_DECIMALS · constant · L8-L8 — pub(crate) const MAX_DECIMALS: usize = 400;
- decimal_digits · function · L17-L60 — pub(crate) fn decimal_digits(a: f64) -> ([u8; 15], i32)
- significant_digits · function · L64-L68 — fn significant_digits(a: f64, n: usize) -> (Vec<u8>, i32)
- is_exact_tie · function · L71-L76 — fn is_exact_tie(a: f64) -> bool
- fixed_from_digits · function · L81-L125 — pub(crate) fn fixed_from_digits(digits: &[u8; 15], exp: i32, decimals: usize) -> (String, String)
- fixed · function · L128-L134 — pub(crate) fn fixed(a: f64, decimals: usize) -> (String, String)
- format_general_fit · function · L138-L193 — pub fn format_general_fit(n: f64, max_chars: usize) -> Option<String>
- tests · module · L196-L234 — mod tests
- rounding · function · L200-L211 — fn rounding()
- general_fit · function · L214-L233 — fn general_fit()
