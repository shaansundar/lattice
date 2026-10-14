//! Downloads a model file safely: resumable, cancellable, size-capped and SHA-256-verified before it is used.
//!
//! The file grows as `<name>.part` in the models folder and is renamed into place only after its size and hash
//! match, so a crash, a cut connection or a full disk never leaves a file the runtime would load. Google Drive's
//! interstitial pages ("can't scan for viruses", "quota exceeded", sign-in) are recognised instead of being saved.
//!
//! Runs on a worker thread (blocking I/O, no async runtime); the UI reads [`Progress`].

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering};
use std::time::Duration;

use sha2::{Digest, Sha256};

use crate::models::{ModelInfo, free_bytes, has_size, human_bytes};

/// What to fetch and how to recognise it.
#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    pub file: String,
    pub bytes: u64,
    pub sha256: String,
    pub urls: Vec<String>,
}

impl Target {
    pub fn for_model(m: &ModelInfo, urls: Vec<String>) -> Target {
        Target { file: m.file.into(), bytes: m.bytes, sha256: m.sha256.into(), urls }
    }
}

pub const PHASE_CONNECTING: u8 = 0;
pub const PHASE_DOWNLOADING: u8 = 1;
pub const PHASE_VERIFYING: u8 = 2;

/// Shared between the worker and the UI.
#[derive(Debug, Default)]
pub struct Progress {
    /// Bytes on disk (downloading) or hashed (verifying).
    pub done: AtomicU64,
    pub total: AtomicU64,
    pub phase: AtomicU8,
    pub cancel: AtomicBool,
}

impl Progress {
    pub fn fraction(&self) -> f32 {
        let total = self.total.load(Ordering::Relaxed);
        if total == 0 { 0.0 } else { (self.done.load(Ordering::Relaxed) as f64 / total as f64).clamp(0.0, 1.0) as f32 }
    }

    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum InstallError {
    Cancelled,
    Failed(String),
}

impl std::fmt::Display for InstallError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallError::Cancelled => f.write_str("download cancelled"),
            InstallError::Failed(m) => f.write_str(m),
        }
    }
}

/// One attempt's outcome: worth retrying the same URL, or move on to the next.
enum Attempt {
    Retry(String),
    NextSource(String),
    Cancelled,
}

/// Tries per URL without progress before moving on (progress resets the count, so a flaky but working connection
/// keeps going).
const MAX_STALLED_TRIES: u32 = 4;
/// A single HTTP call may run this long before it is restarted from where it stopped (stalled connections).
const CALL_TIMEOUT: Duration = Duration::from_secs(20 * 60);
/// Head-room kept free on the disk beyond the file itself.
const DISK_MARGIN: u64 = 256 * 1024 * 1024;
const BUF: usize = 1 << 20;

/// Download `t` into `dir` and return the installed path. Already-installed files (exact size) are returned as is.
pub fn install(t: &Target, dir: &Path, progress: &Progress) -> Result<PathBuf, InstallError> {
    let fail = |m: String| InstallError::Failed(m);
    std::fs::create_dir_all(dir).map_err(|e| fail(format!("can't create the models folder {}: {e}", dir.display())))?;
    let final_path = dir.join(&t.file);
    progress.total.store(t.bytes, Ordering::Relaxed);
    if has_size(&final_path, t.bytes) {
        progress.done.store(t.bytes, Ordering::Relaxed);
        return Ok(final_path);
    }
    if t.urls.is_empty() {
        return Err(fail(
            "Downloads aren't set up in this build yet. Ask whoever gave you Lattice for the model file, then use \"Use a file I already have…\"."
                .into(),
        ));
    }
    let part = dir.join(format!("{}.part", t.file));
    let have = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
    if let Some(free) = free_bytes(dir) {
        let need = t.bytes.saturating_sub(have).saturating_add(DISK_MARGIN);
        if free < need {
            return Err(fail(format!("Not enough disk space: the model needs {} free and this disk has {}.", human_bytes(need), human_bytes(free))));
        }
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_connect(Some(Duration::from_secs(20)))
        .timeout_recv_response(Some(Duration::from_secs(60)))
        .timeout_per_call(Some(CALL_TIMEOUT))
        .max_redirects(10)
        .user_agent(concat!("Lattice/", env!("CARGO_PKG_VERSION")))
        .build()
        .into();
    let mut last_error = String::from("the download failed");
    for url in &t.urls {
        let mut url = url.clone();
        let mut stalled = 0u32;
        let mut confirm_pages = 0u32;
        while stalled < MAX_STALLED_TRIES {
            let before = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
            match fetch(&agent, &url, &part, t.bytes, progress) {
                Ok(()) => return finish(t, &part, &final_path, progress),
                Err(Attempt::Cancelled) => return Err(InstallError::Cancelled),
                Err(Attempt::NextSource(m)) if m.starts_with("confirm:") && confirm_pages < 2 => {
                    // Drive's "can't scan for viruses" page: follow its download form once or twice.
                    confirm_pages += 1;
                    url = m.trim_start_matches("confirm:").to_string();
                }
                Err(Attempt::NextSource(m)) => {
                    last_error = m;
                    break;
                }
                Err(Attempt::Retry(m)) => {
                    last_error = m;
                    let after = std::fs::metadata(&part).map(|m| m.len()).unwrap_or(0);
                    stalled = if after > before { 0 } else { stalled + 1 };
                    if !pause(progress, Duration::from_secs(1u64 << stalled.min(4))) {
                        return Err(InstallError::Cancelled);
                    }
                }
            }
        }
    }
    Err(fail(last_error))
}

/// Sleep in short steps so Cancel stays responsive; false when cancelled.
fn pause(progress: &Progress, d: Duration) -> bool {
    let steps = (d.as_millis() / 100).max(1);
    for _ in 0..steps {
        if progress.cancelled() {
            return false;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    !progress.cancelled()
}

/// One HTTP call appending to `part` from where it stopped.
fn fetch(agent: &ureq::Agent, url: &str, part: &Path, bytes: u64, progress: &Progress) -> Result<(), Attempt> {
    if progress.cancelled() {
        return Err(Attempt::Cancelled);
    }
    progress.phase.store(PHASE_CONNECTING, Ordering::Relaxed);
    let mut have = std::fs::metadata(part).map(|m| m.len()).unwrap_or(0);
    if have > bytes {
        truncate(part)?;
        have = 0;
    }
    if have == bytes {
        return Ok(());
    }
    let mut req = agent.get(url);
    if have > 0 {
        req = req.header("Range", format!("bytes={have}-"));
    }
    let mut resp = req.call().map_err(|e| Attempt::Retry(network_message(&e.to_string())))?;
    let status = resp.status().as_u16();
    let html = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).is_some_and(|v| v.to_ascii_lowercase().contains("text/html"));
    if html {
        let page = resp.body_mut().with_config().limit(512 * 1024).read_to_string().unwrap_or_default();
        return Err(match drive_confirm_url(&page) {
            Some(next) => Attempt::NextSource(format!("confirm:{next}")),
            None => Attempt::NextSource(drive_page_message(&page)),
        });
    }
    let append = match status {
        206 => true,
        200 => false,
        416 => {
            // Range past the end: the part is complete or stale; start over unless it is exactly done.
            truncate(part)?;
            return Err(Attempt::Retry("the server rejected the resume point; starting over".into()));
        }
        404 | 410 => return Err(Attempt::NextSource("The model file isn't available at its download address any more.".into())),
        401 | 403 => {
            return Err(Attempt::NextSource(
                "The download was refused (the file may not be shared publicly, or its daily download limit was reached). Try again later.".into(),
            ));
        }
        429 | 500..=599 => return Err(Attempt::Retry(format!("the download server is busy (HTTP {status})"))),
        _ => return Err(Attempt::NextSource(format!("unexpected answer from the download server (HTTP {status})"))),
    };
    let mut file = if append {
        OpenOptions::new().append(true).open(part).map_err(|e| Attempt::NextSource(disk_message(&e)))?
    } else {
        have = 0;
        File::create(part).map_err(|e| Attempt::NextSource(disk_message(&e)))?
    };
    progress.phase.store(PHASE_DOWNLOADING, Ordering::Relaxed);
    progress.done.store(have, Ordering::Relaxed);
    let remaining = bytes.saturating_sub(have);
    // One byte past the expected size is enough to know the server sent something else.
    let mut reader = resp.body_mut().with_config().limit(remaining.saturating_add(1)).reader();
    let mut buf = vec![0u8; BUF];
    let mut written = have;
    loop {
        if progress.cancelled() {
            let _ = file.flush();
            return Err(Attempt::Cancelled);
        }
        let n = match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) => {
                let _ = file.flush();
                return Err(Attempt::Retry(network_message(&e.to_string())));
            }
        };
        written = written.saturating_add(n as u64);
        if written > bytes {
            drop(file);
            truncate(part)?;
            return Err(Attempt::NextSource("The server sent a different file than expected (it is too large).".into()));
        }
        let chunk = buf.get(..n).unwrap_or(&[]);
        file.write_all(chunk).map_err(|e| Attempt::NextSource(disk_message(&e)))?;
        progress.done.store(written, Ordering::Relaxed);
    }
    file.flush().map_err(|e| Attempt::NextSource(disk_message(&e)))?;
    if written < bytes {
        return Err(Attempt::Retry(format!("the connection closed early ({} of {})", human_bytes(written), human_bytes(bytes))));
    }
    Ok(())
}

/// Hash the finished part file and move it into place; a mismatch deletes it.
fn finish(t: &Target, part: &Path, final_path: &Path, progress: &Progress) -> Result<PathBuf, InstallError> {
    progress.phase.store(PHASE_VERIFYING, Ordering::Relaxed);
    match sha256_file(part, progress)? {
        h if h == t.sha256 => {}
        _ => {
            let _ = std::fs::remove_file(part);
            return Err(InstallError::Failed("The downloaded file was damaged (its checksum doesn't match), so it was deleted. Try again.".into()));
        }
    }
    std::fs::rename(part, final_path).map_err(|e| InstallError::Failed(format!("can't move the model into place: {e}")))?;
    progress.done.store(t.bytes, Ordering::Relaxed);
    Ok(final_path.to_path_buf())
}

/// Verify a file the user already has and copy it into `dir` (the offline path). The source is never modified.
pub fn import(t: &Target, source: &Path, dir: &Path, progress: &Progress) -> Result<PathBuf, InstallError> {
    let fail = |m: String| InstallError::Failed(m);
    let len = std::fs::metadata(source).map_err(|e| fail(format!("can't read {}: {e}", source.display())))?.len();
    if len != t.bytes {
        return Err(fail(format!("That file isn't {} (it is {}, the model is {}).", t.file, human_bytes(len), human_bytes(t.bytes))));
    }
    progress.total.store(t.bytes, Ordering::Relaxed);
    progress.phase.store(PHASE_VERIFYING, Ordering::Relaxed);
    if sha256_file(source, progress)? != t.sha256 {
        return Err(fail(format!("That file isn't a genuine copy of {} (its checksum doesn't match).", t.file)));
    }
    std::fs::create_dir_all(dir).map_err(|e| fail(format!("can't create the models folder {}: {e}", dir.display())))?;
    let final_path = dir.join(&t.file);
    if final_path.as_path() == source {
        return Ok(final_path);
    }
    let part = dir.join(format!("{}.part", t.file));
    std::fs::copy(source, &part).map_err(|e| fail(disk_message(&e)))?;
    std::fs::rename(&part, &final_path).map_err(|e| fail(format!("can't move the model into place: {e}")))?;
    Ok(final_path)
}

pub fn sha256_file(path: &Path, progress: &Progress) -> Result<String, InstallError> {
    let mut f = File::open(path).map_err(|e| InstallError::Failed(format!("can't read {}: {e}", path.display())))?;
    let mut h = Sha256::new();
    let mut buf = vec![0u8; BUF];
    let mut done = 0u64;
    progress.done.store(0, Ordering::Relaxed);
    loop {
        if progress.cancelled() {
            return Err(InstallError::Cancelled);
        }
        let n = f.read(&mut buf).map_err(|e| InstallError::Failed(format!("can't read {}: {e}", path.display())))?;
        if n == 0 {
            break;
        }
        h.update(buf.get(..n).unwrap_or(&[]));
        done = done.saturating_add(n as u64);
        progress.done.store(done, Ordering::Relaxed);
    }
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

fn truncate(part: &Path) -> Result<(), Attempt> {
    File::create(part).map(|_| ()).map_err(|e| Attempt::NextSource(disk_message(&e)))
}

fn disk_message(e: &std::io::Error) -> String {
    match e.kind() {
        std::io::ErrorKind::StorageFull | std::io::ErrorKind::QuotaExceeded => "The disk is full. Free some space and try again.".into(),
        std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::ReadOnlyFilesystem => {
            "Lattice isn't allowed to write to its models folder.".into()
        }
        _ => format!("can't write the model file: {e}"),
    }
}

fn network_message(e: &str) -> String {
    let l = e.to_ascii_lowercase();
    if l.contains("dns") || l.contains("resolve") || l.contains("lookup") {
        "No internet connection (couldn't reach the download server).".into()
    } else if l.contains("timed out") || l.contains("timeout") {
        "The connection timed out.".into()
    } else {
        format!("network error: {}", e.chars().take(200).collect::<String>())
    }
}

/// Drive's "Google Drive can't scan this file for viruses" page carries a form that downloads the file anyway:
/// `<form id="download-form" action="https://drive.usercontent.google.com/download" …>` with hidden inputs.
pub(crate) fn drive_confirm_url(page: &str) -> Option<String> {
    let start = page.find("id=\"download-form\"")?;
    let form_open = page.get(..start)?.rfind("<form")?;
    let form = page.get(form_open..)?;
    let form = form.get(..form.find("</form>").unwrap_or(form.len()))?;
    let action = attr(form.get(..form.find('>')?)?, "action")?;
    if !action.starts_with("https://") {
        return None;
    }
    let mut params = Vec::new();
    let mut rest = form;
    while let Some(i) = rest.find("<input") {
        let tag_all = rest.get(i..)?;
        let end = tag_all.find('>')?;
        let tag = tag_all.get(..end)?;
        if tag.contains("type=\"hidden\"")
            && let (Some(n), Some(v)) = (attr(tag, "name"), attr(tag, "value"))
        {
            params.push(format!("{}={}", encode(&n), encode(&v)));
        }
        rest = tag_all.get(end..)?;
    }
    if params.is_empty() {
        return None;
    }
    Some(format!("{action}?{}", params.join("&")))
}

fn attr(tag: &str, name: &str) -> Option<String> {
    let key = format!("{name}=\"");
    let i = tag.find(&key)? + key.len();
    let v = tag.get(i..)?;
    Some(v.get(..v.find('"')?)?.replace("&amp;", "&"))
}

fn encode(s: &str) -> String {
    s.bytes().map(|b| if b.is_ascii_alphanumeric() || b"-_.~".contains(&b) { (b as char).to_string() } else { format!("%{b:02X}") }).collect()
}

fn drive_page_message(page: &str) -> String {
    let l = page.to_ascii_lowercase();
    if l.contains("quota") || l.contains("too many users") {
        "Google Drive's daily download limit for this file was reached. Try again in a few hours.".into()
    } else if l.contains("accounts.google.com") || l.contains("sign in") || l.contains("request access") {
        "The model file isn't shared publicly on Google Drive.".into()
    } else {
        "The download server returned a web page instead of the model file.".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::TcpListener;
    use std::sync::Arc;
    use std::sync::atomic::AtomicUsize;

    /// What the mock server does for one request: (status, content-type, body, bytes to send before hanging up).
    type Reply = Box<dyn Fn(usize, Option<u64>) -> (u16, &'static str, Vec<u8>, Option<usize>) + Send + Sync>;

    /// A tiny HTTP/1.1 server on a loopback port; returns its base URL and a request counter.
    fn serve(reply: Reply) -> (String, Arc<AtomicUsize>) {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = l.local_addr().unwrap().port();
        let count = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&count);
        std::thread::spawn(move || {
            for stream in l.incoming() {
                let Ok(mut s) = stream else { continue };
                let mut r = BufReader::new(s.try_clone().unwrap());
                let mut range = None;
                let mut line = String::new();
                while r.read_line(&mut line).unwrap_or(0) > 0 {
                    let t = line.trim().to_ascii_lowercase();
                    if t.is_empty() {
                        break;
                    }
                    if let Some(v) = t.strip_prefix("range: bytes=") {
                        range = v.trim_end_matches('-').parse().ok();
                    }
                    line.clear();
                }
                let n = c.fetch_add(1, Ordering::SeqCst);
                let (status, ctype, body, cut) = reply(n, range);
                let head = format!("HTTP/1.1 {status} X\r\nContent-Type: {ctype}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                let _ = s.write_all(head.as_bytes());
                let _ = s.write_all(&body[..cut.unwrap_or(body.len()).min(body.len())]);
            }
        });
        (format!("http://127.0.0.1:{port}"), count)
    }

    fn data() -> Vec<u8> {
        (0..300_000u32).map(|i| (i % 251) as u8).collect()
    }

    fn sha(b: &[u8]) -> String {
        Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
    }

    fn target(urls: Vec<String>, body: &[u8]) -> Target {
        Target { file: "m.gguf".into(), bytes: body.len() as u64, sha256: sha(body), urls }
    }

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("lattice-dl-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn downloads_verifies_and_resumes_after_a_cut() {
        let body = data();
        let b = body.clone();
        // First call hangs up halfway; the second must ask for the rest with Range and gets 206.
        let (url, count) = serve(Box::new(move |n, range| match (n, range) {
            (0, None) => (200, "application/octet-stream", b.clone(), Some(b.len() / 2)),
            (_, Some(from)) => (206, "application/octet-stream", b[from as usize..].to_vec(), None),
            _ => (200, "application/octet-stream", b.clone(), None),
        }));
        let dir = tmp("resume");
        let p = Progress::default();
        let got = install(&target(vec![format!("{url}/m.gguf")], &body), &dir, &p).unwrap();
        assert_eq!(std::fs::read(&got).unwrap(), body);
        assert!(count.load(Ordering::SeqCst) >= 2);
        assert!(!dir.join("m.gguf.part").exists());
        assert_eq!(p.fraction(), 1.0);
        // Installed files are recognised without a network call.
        let again = install(&target(vec!["http://127.0.0.1:1/x".into()], &body), &dir, &Progress::default()).unwrap();
        assert_eq!(again, got);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn wrong_hash_is_deleted_and_reported() {
        let body = data();
        let b = body.clone();
        let (url, _) = serve(Box::new(move |_, _| (200, "application/octet-stream", b.clone(), None)));
        let dir = tmp("hash");
        let mut t = target(vec![url], &body);
        t.sha256 = "0".repeat(64);
        let e = install(&t, &dir, &Progress::default()).unwrap_err();
        assert!(e.to_string().contains("damaged"), "{e}");
        assert!(!dir.join("m.gguf").exists() && !dir.join("m.gguf.part").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn drive_quota_page_is_not_saved_and_next_source_is_tried() {
        let body = data();
        let b = body.clone();
        let (bad, _) = serve(Box::new(|_, _| {
            (200, "text/html; charset=utf-8", b"<html>Too many users have viewed or downloaded this file recently</html>".to_vec(), None)
        }));
        let (good, _) = serve(Box::new(move |_, _| (200, "application/octet-stream", b.clone(), None)));
        let dir = tmp("quota");
        let got = install(&target(vec![bad.clone(), good], &body), &dir, &Progress::default()).unwrap();
        assert_eq!(std::fs::read(got).unwrap(), body);
        let e = install(&target(vec![bad], &body), &tmp("quota2"), &Progress::default()).unwrap_err();
        assert!(e.to_string().contains("daily download limit"), "{e}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn oversized_and_missing_files_fail_cleanly() {
        let body = data();
        let mut big = body.clone();
        big.extend_from_slice(b"extra");
        let (url, _) = serve(Box::new(move |_, _| (200, "application/octet-stream", big.clone(), None)));
        let dir = tmp("big");
        let e = install(&target(vec![url], &body), &dir, &Progress::default()).unwrap_err();
        assert!(e.to_string().contains("too large"), "{e}");
        let (gone, _) = serve(Box::new(|_, _| (404, "application/octet-stream", Vec::new(), None)));
        let e = install(&target(vec![gone], &body), &dir, &Progress::default()).unwrap_err();
        assert!(e.to_string().contains("isn't available"), "{e}");
        let e = install(&target(vec![], &body), &dir, &Progress::default()).unwrap_err();
        assert!(e.to_string().contains("aren't set up"), "{e}");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cancel_stops_and_keeps_the_partial_file() {
        let body = data();
        let b = body.clone();
        let (url, _) = serve(Box::new(move |_, _| (200, "application/octet-stream", b.clone(), None)));
        let dir = tmp("cancel");
        let p = Progress::default();
        p.cancel.store(true, Ordering::Relaxed);
        assert_eq!(install(&target(vec![url], &body), &dir, &p), Err(InstallError::Cancelled));
        assert!(!dir.join("m.gguf").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn import_checks_size_and_hash() {
        let body = data();
        let dir = tmp("import");
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("download.gguf");
        std::fs::write(&src, &body).unwrap();
        let out = dir.join("models");
        let got = import(&target(vec![], &body), &src, &out, &Progress::default()).unwrap();
        assert_eq!(std::fs::read(got).unwrap(), body);
        assert!(src.exists(), "the user's file is left alone");
        let mut t = target(vec![], &body);
        t.sha256 = "f".repeat(64);
        assert!(import(&t, &src, &dir.join("m2"), &Progress::default()).unwrap_err().to_string().contains("genuine"));
        t.bytes += 1;
        assert!(import(&t, &src, &dir.join("m3"), &Progress::default()).unwrap_err().to_string().contains("isn't"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parses_drive_virus_scan_form() {
        let page = r#"<html><body><form id="download-form" action="https://drive.usercontent.google.com/download" method="get">
            <input type="submit" value="Download anyway"/><input type="hidden" name="id" value="abc">
            <input type="hidden" name="export" value="download"><input type="hidden" name="confirm" value="t">
            <input type="hidden" name="uuid" value="u-1&amp;2"></form></body></html>"#;
        assert_eq!(
            drive_confirm_url(page).as_deref(),
            Some("https://drive.usercontent.google.com/download?id=abc&export=download&confirm=t&uuid=u-1%262")
        );
        assert_eq!(drive_confirm_url("<html>nothing</html>"), None);
        assert_eq!(drive_confirm_url(r#"<form id="download-form" action="javascript:x">"#), None);
    }
}
