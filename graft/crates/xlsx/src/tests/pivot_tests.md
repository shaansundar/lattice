# crates/xlsx/src/tests/pivot_tests.rs

- at · function · L15-L17 — fn at(a: &str) -> CellRef
- rr · function · L18-L20 — fn rr(a: &str) -> RangeRef
- date · function · L22-L24 — fn date(y: i64, m: i64, d: i64) -> f64
- field · function · L26-L28 — fn field(name: &str) -> PivotField
- workbook · function · L32-L187 — fn workbook() -> Workbook
- unzip · function · L189-L199 — fn unzip(bytes: &[u8]) -> HashMap<String, Vec<u8>>
- pivots_round_trip · function · L202-L220 — fn pivots_round_trip()
- pivot_package_structure · function · L223-L322 — fn pivot_package_structure()
- pivot_xml_details · function · L325-L349 — fn pivot_xml_details()
- unresolvable_pivot_source_is_skipped · function · L352-L359 — fn unresolvable_pivot_source_is_skipped()
- large_cache_is_written_without_records · function · L362-L391 — fn large_cache_is_written_without_records()
- PT_RELS · constant · L393-L393 — const PT_RELS: &str = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotTable" Target="../pivotTables/pivotTable1.xml"/></Relationships>"#;
- PT_CACHE_RELS · constant · L394-L394 — const PT_CACHE_RELS: &str = r#"<?xml version="1.0"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/pivotCacheDefinition" Target="../pivotCache/pivotCacheDefinition1.xml"/></Relationships>"#;
- with_pivot · function · L396-L406 — fn with_pivot(table: &str, cache: Option<&str>) -> Vec<u8>
- GOOD_CACHE · constant · L408-L408 — const GOOD_CACHE: &str = r#"<pivotCacheDefinition xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><cacheSource type="worksheet"><worksheetSource ref="A1:B3"/></cacheSource><cacheFields count="2"><cacheField name="K"><sharedItems><s v="x"/><n v="2"/><m/><b v="1"/><d v="2024-01-01T00:00:00"/></sharedItems></cacheField><cacheField name="V"><sharedItems/><fieldGroup base="0"><rangePr startNum="0" endNum="10" groupInterval="5"/><groupItems><s v="0-4"/></groupItems></fieldGroup></cacheField></cacheFields></pivotCacheDefinition>"#;
- hostile_pivot_parts_warn_not_panic · function · L411-L449 — fn hostile_pivot_parts_warn_not_panic()
- cache_found_by_workbook_cache_id · function · L452-L481 — fn cache_found_by_workbook_cache_id()
