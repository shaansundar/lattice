//! The `lattice_docs` tool: Lattice's own documentation, embedded at build time, split at headings and ranked with
//! BM25. A new Lattice release updates how-to answers without retraining the model. Same chunking and scoring as
//! the training harness (`lattice_harness/docs_index.py`).

use std::collections::{HashMap, HashSet};

const MAX_CHARS: usize = 1400;
const TOP_K: usize = 3;

/// (file name, contents) in the order the training index used.
pub const DOC_FILES: &[(&str, &str)] = &[
    ("README.md", include_str!("../../../README.md")),
    ("cli.md", include_str!("../../../docs/cli.md")),
    ("mcp.md", include_str!("../../../docs/mcp.md")),
    ("control-protocol.md", include_str!("../../../docs/control-protocol.md")),
    ("formula-input.md", include_str!("../../../docs/formula-input.md")),
    ("cell-pictures.md", include_str!("../../../docs/cell-pictures.md")),
    ("xlsb-import.md", include_str!("../../../docs/xlsb-import.md")),
    ("ods-import.md", include_str!("../../../docs/ods-import.md")),
    ("architecture.md", include_str!("../../../docs/architecture.md")),
];

/// Lowercase runs of [a-z0-9._-] starting with an alphanumeric, trimmed of ._- and at least 2 chars.
pub fn tokens(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();
    let mut out = Vec::new();
    let mut cur = String::new();
    let flush = |cur: &mut String, out: &mut Vec<String>| {
        let t = cur.trim_matches(|c| matches!(c, '.' | '-' | '_'));
        if t.chars().count() > 1 {
            out.push(t.to_string());
        }
        cur.clear();
    };
    for c in lower.chars() {
        let alnum = c.is_ascii_lowercase() || c.is_ascii_digit();
        if alnum || (!cur.is_empty() && matches!(c, '.' | '_' | '-')) {
            cur.push(c);
        } else if !cur.is_empty() {
            flush(&mut cur, &mut out);
        }
    }
    if !cur.is_empty() {
        flush(&mut cur, &mut out);
    }
    out
}

fn python_strip(s: &str) -> &str {
    s.trim_matches(char::is_whitespace)
}

/// Split one markdown file at `#`..`###` headings into (title, body) chunks of at most MAX_CHARS.
pub fn chunk_markdown(name: &str, text: &str) -> Vec<(String, String)> {
    let mut chunks = Vec::new();
    let mut path: Vec<String> = Vec::new();
    let mut body: Vec<&str> = Vec::new();
    let flush = |path: &[String], body: &[&str], chunks: &mut Vec<(String, String)>| {
        let joined = body.join("\n");
        let b = python_strip(&joined);
        if b.is_empty() {
            return;
        }
        let title = std::iter::once(name.to_string()).chain(path.iter().cloned()).collect::<Vec<_>>().join(" › ");
        let mut cur = String::new();
        for p in b.split("\n\n") {
            if !cur.is_empty() && cur.chars().count() + p.chars().count() + 2 > MAX_CHARS {
                chunks.push((title.clone(), std::mem::take(&mut cur)));
            }
            cur = if cur.is_empty() { p.to_string() } else { format!("{cur}\n\n{p}") };
        }
        if !cur.is_empty() {
            chunks.push((title, cur.chars().take(MAX_CHARS * 2).collect()));
        }
    };
    for line in text.lines() {
        let hashes = line.chars().take_while(|c| *c == '#').count();
        let rest = line.get(hashes..).unwrap_or("");
        let is_heading = (1..=3).contains(&hashes) && rest.starts_with(char::is_whitespace);
        if is_heading {
            flush(&path, &body, &mut chunks);
            body.clear();
            path = if hashes > 1 {
                let mut p: Vec<String> = path.iter().take(hashes.saturating_sub(2)).cloned().collect();
                p.push(python_strip(rest).to_string());
                p
            } else {
                Vec::new()
            };
        } else {
            body.push(line);
        }
    }
    flush(&path, &body, &mut chunks);
    chunks
}

pub struct DocsIndex {
    chunks: Vec<(String, String)>,
    toks: Vec<Vec<String>>,
    df: HashMap<String, usize>,
    avg: f64,
}

impl DocsIndex {
    pub fn new(chunks: Vec<(String, String)>) -> Self {
        let toks: Vec<Vec<String>> = chunks.iter().map(|(t, b)| tokens(&format!("{t} {b}"))).collect();
        let mut df = HashMap::new();
        for ts in &toks {
            for w in ts.iter().collect::<HashSet<_>>() {
                *df.entry(w.clone()).or_insert(0) += 1;
            }
        }
        let avg = toks.iter().map(Vec::len).sum::<usize>() as f64 / toks.len().max(1) as f64;
        DocsIndex { chunks, toks, df, avg }
    }

    /// The documentation bundled into this build.
    pub fn bundled() -> Self {
        Self::new(DOC_FILES.iter().flat_map(|(n, t)| chunk_markdown(n, t)).collect())
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn search(&self, query: &str) -> Vec<&(String, String)> {
        let (k1, b) = (1.2_f64, 0.75_f64);
        let q = tokens(query);
        let n = self.chunks.len() as f64;
        let mut scored: Vec<(f64, usize)> = Vec::new();
        for (i, ts) in self.toks.iter().enumerate() {
            let mut tf: HashMap<&str, f64> = HashMap::new();
            for w in ts {
                *tf.entry(w.as_str()).or_insert(0.0) += 1.0;
            }
            let mut s = 0.0;
            for w in &q {
                if let Some(&f) = tf.get(w.as_str()) {
                    let d = *self.df.get(w).unwrap_or(&0) as f64;
                    let idf = (1.0 + (n - d + 0.5) / (d + 0.5)).ln();
                    s += idf * f * (k1 + 1.0) / (f + k1 * (1.0 - b + b * ts.len() as f64 / self.avg));
                }
            }
            if s > 0.0 {
                scored.push((s, i));
            }
        }
        scored.sort_by(|a, c| c.0.total_cmp(&a.0).then(a.1.cmp(&c.1)));
        scored.into_iter().take(TOP_K).filter_map(|(_, i)| self.chunks.get(i)).collect()
    }

    /// Observation text for `lattice_docs(search)`.
    pub fn render(&self, query: &str) -> String {
        let hits = self.search(query);
        if hits.is_empty() {
            return "No documentation matches; try other words.".into();
        }
        hits.iter().map(|(t, b)| format!("## {t}\n{b}")).collect::<Vec<_>>().join("\n\n")
    }
}
