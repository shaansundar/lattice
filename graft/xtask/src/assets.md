# xtask/src/assets.rs

- ASSET_EXT · constant · L15-L16 — const ASSET_EXT: &[&str] =
- EXAMPLE_EXT · constant · L18-L18 — const EXAMPLE_EXT: &[&str] = &["csv", "tsv", "json"];
- ASSET_DIRS · constant · L20-L20 — const ASSET_DIRS: &[&str] = &["assets/", "docs/brand/", "docs/images/"];
- ext · function · L22-L24 — fn ext(path: &str) -> String
- is_asset · function · L26-L32 — pub fn is_asset(path: &str) -> bool
- missing · function · L35-L37 — pub fn missing(files: &[String], attribution: &str) -> Vec<String>
- run · function · L39-L57 — pub fn run(root: &Path) -> Result<(), String>
- tests · module · L60-L86 — mod tests
- detects_unattributed_assets · function · L64-L74 — fn detects_unattributed_assets()
- classifies_paths · function · L77-L85 — fn classifies_paths()
