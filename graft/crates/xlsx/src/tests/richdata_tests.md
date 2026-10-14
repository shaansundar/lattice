# crates/xlsx/src/tests/richdata_tests.rs

- at · function · L12-L14 — fn at(value: &str) -> CellRef
- png · function · L17-L45 — fn png() -> Vec<u8>
- chunk · function · L18-L30 — fn chunk(out: &mut Vec<u8>, kind: &[u8], body: &[u8])
- jpeg · function · L48-L68 — fn jpeg() -> Vec<u8>
- segment · function · L49-L53 — fn segment(data: &mut Vec<u8>, marker: u8, payload: &[u8])
- picture · function · L70-L72 — fn picture(data: Vec<u8>, mime: &str, alt: &str) -> CellPicture
- independent_parts · function · L76-L128 — fn independent_parts(chain: bool) -> Vec<(String, Vec<u8>)>
- zip · function · L130-L132 — fn zip(parts: &[(String, Vec<u8>)]) -> Vec<u8>
- replace · function · L134-L138 — fn replace(parts: &mut [(String, Vec<u8>)], name: &str, from: &str, to: &str)
- reads_independent_indices_reordered_keys_and_both_relationship_layouts · function · L141-L152 — fn reads_independent_indices_reordered_keys_and_both_relationship_layouts()
- pictures_roundtrip_across_sheets_with_and_without_dynamic_arrays · function · L155-L196 — fn pictures_roundtrip_across_sheets_with_and_without_dynamic_arrays()
- broken_picture_references_keep_the_scalar_fallback_with_warnings · function · L199-L219 — fn broken_picture_references_keep_the_scalar_fallback_with_warnings()
- hostile_image_bytes_and_dimensions_are_bounded · function · L222-L237 — fn hostile_image_bytes_and_dimensions_are_bounded()
- oversized_entry · function · L241-L254 — fn oversized_entry(mut bytes: Vec<u8>, path: &str, size: u32) -> Vec<u8>
- picture_metadata_limits_and_relationship_read_errors_do_not_block_workbooks · function · L257-L272 — fn picture_metadata_limits_and_relationship_read_errors_do_not_block_workbooks()
