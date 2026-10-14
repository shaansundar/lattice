# xtask/src/main.rs

- assets · module · L8-L8 — mod assets;
- ico · module · L9-L9 — mod ico;
- layers · module · L10-L10 — mod layers;
- parity · module · L12-L12 — mod parity;
- stats · module · L13-L13 — mod stats;
- version · module · L14-L14 — mod version;
- USAGE · constant · L19-L35 — const USAGE: &str = "\
- main · function · L37-L62 — fn main() -> ExitCode
- root · function · L65-L68 — pub fn root() -> PathBuf
- cargo · function · L70-L74 — pub fn cargo() -> Command
- run · function · L76-L80 — fn run(mut cmd: Command, what: &str) -> Result<(), String>
- metadata · function · L82-L88 — pub fn metadata() -> Result<serde_json::Value, String>
- cmd_layers · function · L90-L122 — fn cmd_layers() -> Result<(), String>
- wasm_set · function · L126-L136 — fn wasm_set() -> Result<Vec<String>, String>
- cmd_wasm · function · L138-L154 — fn cmd_wasm() -> Result<(), String>
- cmd_ci · function · L156-L216 — fn cmd_ci() -> Result<(), String>
- Step · type · L157-L157 — type Step = (&'static str, Box<dyn Fn() -> Result<(), String>>);
