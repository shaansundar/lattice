# crates/model/src/cellmap.rs

- serialize · function · L10-L12 — pub fn serialize<S: Serializer, V: Serialize>(map: &BTreeMap<CellRef, V>, s: S) -> Result<S::Ok, S::Error>
- deserialize · function · L14-L22 — pub fn deserialize<'de, D: Deserializer<'de>, V: Deserialize<'de>>(d: D) -> Result<BTreeMap<CellRef, V>, D::Error>
- tests · module · L25-L51 — mod tests
- comments_and_hyperlinks_survive_json · function · L30-L50 — fn comments_and_hyperlinks_survive_json()
