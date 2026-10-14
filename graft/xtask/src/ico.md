# xtask/src/ico.rs

- png_size · function · L9-L17 — pub fn png_size(png: &[u8]) -> Result<(u32, u32), String>
- SIG · constant · L10-L10 — const SIG: &[u8] = b"\x89PNG\r\n\x1a\n";
- pack · function · L20-L46 — pub fn pack(pngs: &[Vec<u8>]) -> Result<Vec<u8>, String>
- run · function · L48-L57 — pub fn run(args: &[&str]) -> Result<(), String>
- tests · module · L60-L104 — mod tests
- fake_png · function · L63-L69 — fn fake_png(size: u32) -> Vec<u8>
- packs_directory_and_payloads · function · L72-L86 — fn packs_directory_and_payloads()
- rejects_bad_input · function · L89-L94 — fn rejects_bad_input()
- committed_icon_is_valid · function · L97-L103 — fn committed_icon_is_valid()
