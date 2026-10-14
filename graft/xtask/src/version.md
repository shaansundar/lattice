# xtask/src/version.rs

- read · function · L9-L16 — pub fn read(manifest: &str) -> Result<String, String>
- replace · function · L19-L38 — pub fn replace(manifest: &str, new: &str) -> Result<String, String>
- validate · function · L41-L57 — pub fn validate(v: &str) -> Result<(), String>
- find_line · function · L60-L76 — fn find_line(manifest: &str) -> Result<(usize, &str), String>
- run · function · L78-L107 — pub fn run(root: &Path, args: &[&str]) -> Result<(), String>
- tests · module · L110-L152 — mod tests
- MANIFEST · constant · L113-L113 — const MANIFEST: &str = "[workspace]\nmembers = [\"a\"]\n\n[workspace.package]\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace.dependencies]\nfoo = { version = \"1\" }\n";
- reads_the_workspace_version · function · L116-L119 — fn reads_the_workspace_version()
- replaces_only_the_workspace_version · function · L122-L127 — fn replaces_only_the_workspace_version()
- keeps_crlf_line_endings · function · L130-L134 — fn keeps_crlf_line_endings()
- validates_versions · function · L137-L145 — fn validates_versions()
- the_real_manifest_has_a_valid_version · function · L148-L151 — fn the_real_manifest_has_a_valid_version()
