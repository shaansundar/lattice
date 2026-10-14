//! The assistant's downloadable models: what each tier is, where it comes from, where it is installed and which one
//! suits this machine.
//!
//! Model files are data, never code: they live in a per-user folder outside the (signed) app bundle and are
//! accepted only when their size and SHA-256 match the pinned values below. Where they are downloaded from is a list
//! of [`Source`]s, so moving them to another host means changing that list and nothing else.

use std::path::{Path, PathBuf};

use crate::router::{Tier, total_ram_bytes};

/// Where a model file can be downloaded from, tried in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// A Google Drive file shared as "Anyone with the link" (`file_id` is the id in its share URL).
    Drive { file_id: &'static str },
    /// A plain HTTPS URL.
    Url(&'static str),
}

impl Source {
    /// The first URL to request. Drive's `confirm=t` skips its "can't scan this file for viruses" page for large
    /// files; the downloader still handles that page if it appears.
    pub fn url(&self) -> String {
        match self {
            Source::Drive { file_id } => format!("https://drive.usercontent.google.com/download?id={file_id}&export=download&confirm=t"),
            Source::Url(u) => (*u).to_string(),
        }
    }
}

/// Everything the setup screen shows about a model, and what the installer checks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelInfo {
    pub tier: Tier,
    /// Short name on the card.
    pub name: &'static str,
    pub file: &'static str,
    /// Exact file size in bytes.
    pub bytes: u64,
    /// Lower-case hex SHA-256 of the file.
    pub sha256: &'static str,
    /// Memory the machine should have for this model to run comfortably.
    pub min_ram_gb: u32,
    pub summary: &'static str,
    pub pros: &'static [&'static str],
    pub cons: &'static [&'static str],
    pub sources: &'static [Source],
}

/// Google Drive ids of the published models ("Anyone with the link"). Empty until the files are shared; the
/// installer then explains that downloads aren't available in this build.
const E4B_DRIVE_ID: &str = "1Ffw5t7FZeiZsnencJDULvhHMgnJK_P1F";
const E2B_DRIVE_ID: &str = "1IKntO5EwxrYNz_pznSSiimoI34NeqQbC";

const E4B_SOURCES: &[Source] = &[Source::Drive { file_id: E4B_DRIVE_ID }];
const E2B_SOURCES: &[Source] = &[Source::Drive { file_id: E2B_DRIVE_ID }];

pub const MODELS: [ModelInfo; 2] = [
    ModelInfo {
        tier: Tier::E4B,
        name: "Standard (E4B)",
        file: "lattice-e4b-Q4_K_M.gguf",
        bytes: 5_302_273_952,
        sha256: "05782c6a11b04bc881c8d4cf14b2ee94994e79eeb330bfb49711fb60e3887907",
        min_ram_gb: 16,
        summary: "The more capable model, for machines with 16 GB of memory or more.",
        pros: &[
            "Most accurate: better how-to answers (67% vs 53%) and real-world tasks",
            "Keeps full math ability",
            "Fast on Apple Silicon and other GPUs",
        ],
        cons: &["5.3 GB download", "Uses about 8 GB of memory without a GPU", "About half the speed of Compact on a CPU"],
        sources: E4B_SOURCES,
    },
    ModelInfo {
        tier: Tier::E2B,
        name: "Compact (E2B)",
        file: "lattice-e2b-Q4_K_M.gguf",
        bytes: 3_416_119_872,
        sha256: "a03f6983e9f4013abd02904aee039537bdfc75181da761837622d47c0f27f8b9",
        min_ram_gb: 8,
        summary: "A lighter model that runs on 8 GB laptops without a GPU.",
        pros: &["About twice as fast", "Uses about 4–5 GB of memory", "Nearly as good at changing workbooks (87% vs 90%)"],
        cons: &["Weaker at how-to answers and harder math", "3.4 GB download"],
        sources: E2B_SOURCES,
    },
];

pub fn info(tier: Tier) -> &'static ModelInfo {
    match tier {
        Tier::E4B => &MODELS[0],
        Tier::E2B => &MODELS[1],
    }
}

impl Tier {
    /// `"e4b"` / `"e2b"`, as commands and settings spell it.
    pub fn id(self) -> &'static str {
        match self {
            Tier::E4B => "e4b",
            Tier::E2B => "e2b",
        }
    }

    pub fn parse(s: &str) -> Option<Tier> {
        match s.trim().to_ascii_lowercase().as_str() {
            "e4b" => Some(Tier::E4B),
            "e2b" => Some(Tier::E2B),
            _ => None,
        }
    }

    pub fn other(self) -> Tier {
        if self == Tier::E4B { Tier::E2B } else { Tier::E4B }
    }
}

/// Download URLs for `tier`, in order. `LATTICE_ASSISTANT_MODEL_URL` (a base URL; the file name is appended) is tried
/// first when set: a mirror, a local test server, or the future licensing backend.
pub fn urls(tier: Tier) -> Vec<String> {
    let m = info(tier);
    let mut out = Vec::new();
    if let Ok(base) = std::env::var("LATTICE_ASSISTANT_MODEL_URL")
        && !base.trim().is_empty()
    {
        out.push(format!("{}/{}", base.trim().trim_end_matches('/'), m.file));
    }
    for s in m.sources {
        if let Source::Drive { file_id } = s
            && file_id.is_empty()
        {
            continue;
        }
        out.push(s.url());
    }
    out
}

/// The per-user folder downloaded models go to (created on demand): `~/Library/Application Support/Lattice/
/// assistant/models` on macOS, `%LOCALAPPDATA%\Lattice\assistant\models` on Windows, `$XDG_DATA_HOME/lattice/
/// assistant/models` (default `~/.local/share`) elsewhere. `LATTICE_ASSISTANT_MODELS_DIR` overrides it.
pub fn models_dir() -> Option<PathBuf> {
    if let Some(d) = std::env::var_os("LATTICE_ASSISTANT_MODELS_DIR").filter(|d| !d.is_empty()) {
        return Some(PathBuf::from(d));
    }
    let home = || std::env::var_os("HOME").filter(|h| !h.is_empty()).map(PathBuf::from);
    if cfg!(target_os = "macos") {
        return home().map(|h| h.join("Library/Application Support/Lattice/assistant/models"));
    }
    if cfg!(windows) {
        return std::env::var_os("LOCALAPPDATA").filter(|d| !d.is_empty()).map(|d| PathBuf::from(d).join("Lattice\\assistant\\models"));
    }
    let data = std::env::var_os("XDG_DATA_HOME").filter(|d| !d.is_empty()).map(PathBuf::from).or_else(|| home().map(|h| h.join(".local/share")))?;
    Some(data.join("lattice/assistant/models"))
}

/// The installed model file for `tier`: in the models folder or next to the bundled runtime (developer and
/// pre-packaged layouts). A file counts only when its size is exact; the hash was checked when it was installed.
pub fn installed(tier: Tier) -> Option<PathBuf> {
    let m = info(tier);
    let mut dirs: Vec<PathBuf> = models_dir().into_iter().collect();
    dirs.extend(crate::install::search_dirs());
    dirs.into_iter().map(|d| d.join(m.file)).find(|p| has_size(p, m.bytes))
}

pub(crate) fn has_size(p: &Path, bytes: u64) -> bool {
    std::fs::metadata(p).map(|md| md.is_file() && md.len() == bytes).unwrap_or(false)
}

/// The model the setup screen badges as best for this machine, and why.
#[derive(Debug, Clone, PartialEq)]
pub struct Recommendation {
    pub tier: Tier,
    pub reason: String,
}

const GIB: u64 = 1 << 30;

/// E4B with at least ~15 GiB of memory (on Apple Silicon the GPU shares it), E2B otherwise.
pub fn recommend() -> Recommendation {
    recommend_for(total_ram_bytes(), cfg!(all(target_os = "macos", target_arch = "aarch64")))
}

pub fn recommend_for(ram: Option<u64>, apple_gpu: bool) -> Recommendation {
    let Some(ram) = ram else {
        return Recommendation {
            tier: Tier::E2B,
            reason: "Couldn't read this computer's memory size, so the lighter model is the safe choice.".into(),
        };
    };
    let gb = (ram.saturating_add(GIB / 2)) / GIB;
    let gpu = if apple_gpu { " and an Apple Silicon GPU" } else { "" };
    if ram >= 15 * GIB {
        Recommendation { tier: Tier::E4B, reason: format!("This computer has {gb} GB of memory{gpu}: enough for the more capable model.") }
    } else if ram >= 7 * GIB {
        Recommendation {
            tier: Tier::E2B,
            reason: format!("This computer has {gb} GB of memory{gpu}; the compact model leaves room for your other apps."),
        }
    } else {
        Recommendation { tier: Tier::E2B, reason: format!("This computer has {gb} GB of memory; even the compact model may be slow here.") }
    }
}

/// The model the user picked last (stored next to the models), if it is still installed.
pub fn preferred() -> Option<Tier> {
    let dir = models_dir()?;
    let s = std::fs::read_to_string(dir.join("selected")).ok()?;
    Tier::parse(&s)
}

pub fn set_preferred(tier: Tier) -> Result<(), String> {
    let dir = models_dir().ok_or("no folder for the assistant's models on this system")?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    std::fs::write(dir.join("selected"), tier.id()).map_err(|e| format!("cannot save the model choice: {e}"))
}

/// Which installed model to run: `LATTICE_ASSISTANT_TIER`, then the user's pick, then the recommendation, then any.
pub fn choose() -> Option<(Tier, PathBuf)> {
    let forced = std::env::var("LATTICE_ASSISTANT_TIER").ok().and_then(|v| Tier::parse(&v));
    let mut order: Vec<Tier> = forced.into_iter().chain(preferred()).collect();
    order.push(recommend().tier);
    order.push(Tier::E4B);
    order.push(Tier::E2B);
    order.into_iter().find_map(|t| installed(t).map(|p| (t, p)))
}

/// Bytes free on the volume holding `dir`, when it can be read (`df`; Windows reports `None` and the download
/// relies on the write error instead).
pub fn free_bytes(dir: &Path) -> Option<u64> {
    if cfg!(windows) {
        return None;
    }
    let out = std::process::Command::new("df").arg("-Pk").arg(dir).output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let kb: u64 = text.lines().nth(1)?.split_whitespace().nth(3)?.parse().ok()?;
    Some(kb.saturating_mul(1024))
}

/// "5.3 GB"-style size for the UI.
pub fn human_bytes(b: u64) -> String {
    let gb = b as f64 / 1e9;
    if gb >= 1.0 { format!("{gb:.1} GB") } else { format!("{:.0} MB", b as f64 / 1e6) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recommendation_follows_memory() {
        assert_eq!(recommend_for(Some(64 * GIB), true).tier, Tier::E4B);
        assert!(recommend_for(Some(64 * GIB), true).reason.contains("64 GB"));
        assert_eq!(recommend_for(Some(16 * GIB), false).tier, Tier::E4B);
        assert_eq!(recommend_for(Some(8 * GIB), false).tier, Tier::E2B);
        assert!(recommend_for(Some(4 * GIB), false).reason.contains("slow"));
        assert_eq!(recommend_for(None, false).tier, Tier::E2B);
    }

    #[test]
    fn catalog_is_consistent() {
        for t in [Tier::E4B, Tier::E2B] {
            let m = info(t);
            assert_eq!(m.tier, t);
            assert_eq!(m.file, t.model_file());
            assert_eq!(m.sha256.len(), 64);
            assert!(m.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
            assert_eq!(Tier::parse(t.id()), Some(t));
            assert_eq!(t.other().other(), t);
        }
        assert_eq!(Tier::parse(" E4B "), Some(Tier::E4B));
        assert_eq!(Tier::parse("gpt"), None);
        assert_eq!(human_bytes(5_302_273_952), "5.3 GB");
        assert_eq!(human_bytes(42_000_000), "42 MB");
    }

    #[test]
    fn drive_sources_use_the_direct_download_host() {
        let u = Source::Drive { file_id: "abc" }.url();
        assert!(u.starts_with("https://drive.usercontent.google.com/download?id=abc"));
        assert!(u.contains("confirm=t"));
    }
}
