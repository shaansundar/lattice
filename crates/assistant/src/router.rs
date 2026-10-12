//! Thinking router and model tier.
//!
//! Thinking (Gemma's reasoning channel) makes multi-step spreadsheet work and math much more accurate (on the agent
//! dev set: 24.7% → 47.3%) but costs roughly a thousand extra tokens per step, which is slow on a laptop CPU. Quick
//! formula help, data look-ups and how-to questions run without it.

/// Where a request came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Entry {
    /// The assistant pane (conversation).
    Chat,
    /// Inline help from the formula bar or a cell: always fast.
    FormulaBar,
}

const HOW_TO: [&str; 9] = ["how do i", "how can i", "where is", "where's", "where do i", "what does", "is there a shortcut", "lattice-cli", "mcp"];
const MATH: [&str; 10] =
    ["solve", "prove", "probability", "equation", "derivative", "integral", "how many ways", "compound interest", "amortiz", "optimi"];
const MULTI_STEP: [&str; 10] =
    [" then ", "and then", "for each", "step by step", "build a", "model the", "forecast", "reconcile", "clean up", "restructure"];

/// Whether this request should use the thinking channel.
pub fn use_thinking(entry: Entry, request: &str) -> bool {
    if entry == Entry::FormulaBar {
        return false;
    }
    let r = request.to_lowercase();
    if HOW_TO.iter().any(|k| r.starts_with(k) || r.contains(&format!(" {k}"))) && r.chars().count() < 200 {
        return false;
    }
    MATH.iter().any(|k| r.contains(k)) || MULTI_STEP.iter().any(|k| r.contains(k)) || r.chars().count() > 280
}

/// Which bundled model this machine runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tier {
    /// Gemma-4-E4B fine-tune: 16 GB+ or a GPU.
    E4B,
    /// Gemma-4-E2B fine-tune: 8 GB, CPU-only machines.
    E2B,
}

impl Tier {
    pub fn model_file(self) -> &'static str {
        match self {
            Tier::E4B => "lattice-e4b-Q4_K_M.gguf",
            Tier::E2B => "lattice-e2b-Q4_K_M.gguf",
        }
    }
}

/// E4B with at least ~15 GiB of RAM (Apple Silicon shares it with the GPU), E2B otherwise. `LATTICE_ASSISTANT_TIER`
/// (`e4b`/`e2b`) overrides.
pub fn detect_tier() -> Tier {
    match std::env::var("LATTICE_ASSISTANT_TIER").map(|v| v.to_ascii_lowercase()).as_deref() {
        Ok("e4b") => return Tier::E4B,
        Ok("e2b") => return Tier::E2B,
        _ => {}
    }
    crate::models::recommend().tier
}

/// GPU layers to offload: all on Apple Silicon (Metal), none elsewhere unless `LATTICE_ASSISTANT_GPU_LAYERS` is set.
pub fn gpu_layers() -> u32 {
    if let Some(n) = std::env::var("LATTICE_ASSISTANT_GPU_LAYERS").ok().and_then(|v| v.parse().ok()) {
        return n;
    }
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) { 99 } else { 0 }
}

/// Physical memory, when the OS reports it.
pub fn total_ram_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let s = std::fs::read_to_string("/proc/meminfo").ok()?;
        let kb: u64 = s.lines().find(|l| l.starts_with("MemTotal:"))?.split_whitespace().nth(1)?.parse().ok()?;
        return Some(kb.saturating_mul(1024));
    }
    #[cfg(target_os = "macos")]
    {
        let out = std::process::Command::new("/usr/sbin/sysctl").args(["-n", "hw.memsize"]).output().ok()?;
        return String::from_utf8_lossy(&out.stdout).trim().parse().ok();
    }
    #[cfg(target_os = "windows")]
    {
        let out = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", "(Get-CimInstance Win32_ComputerSystem).TotalPhysicalMemory"])
            .output()
            .ok()?;
        return String::from_utf8_lossy(&out.stdout).trim().parse().ok();
    }
    #[allow(unreachable_code)]
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_quick_requests_fast_and_heavy_ones_to_thinking() {
        for fast in ["Bold the header row.", "Where is Remove Duplicates?", "How do I freeze the top row?", "What's the total revenue?"] {
            assert!(!use_thinking(Entry::Chat, fast), "{fast}");
        }
        for slow in ["Sort by region, then for each region add a subtotal row", "Solve for the break-even price", &"x".repeat(300)] {
            assert!(use_thinking(Entry::Chat, slow), "{slow}");
        }
        assert!(!use_thinking(Entry::FormulaBar, "solve this step by step"));
    }
}
