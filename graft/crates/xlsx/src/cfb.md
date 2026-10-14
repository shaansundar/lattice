# crates/xlsx/src/cfb.rs

- SIGNATURE · constant · L7-L7 — pub(crate) const SIGNATURE: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
- END_OF_CHAIN · constant · L9-L9 — const END_OF_CHAIN: u32 = 0xFFFF_FFFE;
- MAX_REG_SECT · constant · L11-L11 — const MAX_REG_SECT: u32 = 0xFFFF_FFFA;
- MAX_DIR_SECTORS · constant · L13-L13 — const MAX_DIR_SECTORS: usize = 1024;
- MAX_DIFAT_SECTORS · constant · L15-L15 — const MAX_DIFAT_SECTORS: usize = 1024;
- ENCRYPTED_STREAMS · constant · L18-L18 — const ENCRYPTED_STREAMS: [&str; 2] = ["EncryptedPackage", "EncryptionInfo"];
- u16_at · function · L20-L23 — fn u16_at(b: &[u8], at: usize) -> Option<u16>
- u32_at · function · L25-L28 — fn u32_at(b: &[u8], at: usize) -> Option<u32>
- is_encrypted · function · L33-L38 — pub(crate) fn is_encrypted(bytes: &[u8]) -> bool
- contains_utf16 · function · L40-L43 — fn contains_utf16(hay: &[u8], needle: &str) -> bool
- directory_names · function · L46-L98 — fn directory_names(b: &[u8]) -> Option<Vec<String>>
- tests · module · L101-L166 — pub(crate) mod tests
- build · function · L106-L139 — pub(crate) fn build(streams: &[&str]) -> Vec<u8>
- reads_directory_names · function · L142-L147 — fn reads_directory_names()
- hostile_headers_never_panic · function · L150-L165 — fn hostile_headers_never_panic()
