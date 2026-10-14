# crates/xlsx/src/tests/ods_tests.rs

- MIME · constant · L8-L8 — const MIME: &[u8] = b"application/vnd.oasis.opendocument.spreadsheet";
- MANIFEST · constant · L9-L9 — const MANIFEST: &[u8] = br#"<manifest:manifest xmlns:manifest="urn:oasis:names:tc:opendocument:xmlns:manifest:1.0"><manifest:file-entry manifest:full-path="/" manifest:media-type="application/vnd.oasis.opendocument.spreadsheet"/><manifest:file-entry manifest:full-path="content.xml" manifest:media-type="text/xml"/></manifest:manifest>"#;
- content · function · L11-L15 — fn content(tables: &str) -> String
- ods · function · L16-L18 — fn ods(tables: &str) -> Vec<u8>
- row · function · L19-L21 — fn row(cells: &str) -> Vec<u8>
- text · function · L22-L24 — fn text(s: &str) -> Value
- ods_namespace_declaration_bomb_is_rejected · function · L29-L40 — fn ods_namespace_declaration_bomb_is_rejected()
- ods_typed_values_and_formula_caches_are_constants_and_survive_xlsx · function · L43-L79 — fn ods_typed_values_and_formula_caches_are_constants_and_survive_xlsx()
- ods_ordered_text_preserves_paragraphs_runs_entities_spaces_tabs_and_breaks · function · L82-L91 — fn ods_ordered_text_preserves_paragraphs_runs_entities_spaces_tabs_and_breaks()
- ods_repeats_merges_sheet_names_and_empty_tail_remain_sparse · function · L94-L112 — fn ods_repeats_merges_sheet_names_and_empty_tail_remain_sparse()
- ods_dates_and_durations_use_formats_and_unsupported_values_stay_text · function · L115-L138 — fn ods_dates_and_durations_use_formats_and_unsupported_values_stay_text()
- ods_namespaces_are_resolved_not_inferred_from_prefix_or_local_name · function · L141-L152 — fn ods_namespaces_are_resolved_not_inferred_from_prefix_or_local_name()
- ods_rejects_bad_packages_encryption_and_truncated_xml · function · L155-L179 — fn ods_rejects_bad_packages_encryption_and_truncated_xml()
- ods_expansion_bounds_stop_small_repeat_bombs · function · L182-L203 — fn ods_expansion_bounds_stop_small_repeat_bombs()
- ods_sniff_requires_exact_bounded_mimetype · function · L206-L212 — fn ods_sniff_requires_exact_bounded_mimetype()
- ods_invalid_sheet_names_are_made_safe_for_xlsx · function · L215-L226 — fn ods_invalid_sheet_names_are_made_safe_for_xlsx()
- ods_extension_error_cache_never_imports_its_placeholder_zero · function · L229-L236 — fn ods_extension_error_cache_never_imports_its_placeholder_zero()
- ods_blank_repeats_past_the_sheet_edge_are_clipped · function · L239-L254 — fn ods_blank_repeats_past_the_sheet_edge_are_clipped()
