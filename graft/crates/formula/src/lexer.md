# crates/formula/src/lexer.rs

- Tok · enum · L8-L32 — pub enum Tok
- Token · struct · L35-L39 — pub struct Token
- LexError · struct · L43-L46 — pub struct LexError
- is_word_char · function · L48-L50 — fn is_word_char(c: char) -> bool
- tokenize · function · L53-L55 — pub fn tokenize(src: &str) -> Result<Vec<Token>, LexError>
- tokenize_locale · function · L58-L328 — pub fn tokenize_locale(src: &str, locale: crate::FormulaLocale) -> Result<Vec<Token>, LexError>
