//! The Assistant task pane (desktop only): a conversation with a local model, which works on the open workbook
//! through Lattice's own tools. Each request is one undo step; file, app and window actions and sheet deletion ask
//! first.
//!
//! The first time it opens without a model, the pane shows the setup screen: both models with what they are good
//! and bad at, the one that suits this computer badged, and Download / Cancel / Remove / Use controls. Downloads
//! run on worker threads (resumable, SHA-256-verified); when a model is ready the runtime sidecar starts and the
//! pane turns into the conversation. Every control is a command (`assistant.install`, `assistant.cancelInstall`,
//! `assistant.removeModel`, `assistant.useModel`, `assistant.importModel`, `assistant.models`,
//! `assistant.manageModels`), so the control channel and MCP reach them too.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::Duration;

use egui::{RichText, Ui, vec2};
use lattice_assistant::agent::{Event, Run};
use lattice_assistant::download::{self, InstallError, PHASE_CONNECTING, PHASE_VERIFYING, Progress, Target};
use lattice_assistant::install::{Launched, MISSING_RUNTIME, launch, locate_server, pid_file};
use lattice_assistant::models::{self, ModelInfo, Recommendation, human_bytes};
use lattice_assistant::router::{Entry, Tier, use_thinking};
use lattice_assistant::runtime::{Generator, LlamaServer};
use lattice_assistant::{Harness, with_session};
use serde_json::{Value as Json, json};

use crate::SheetApp;
use crate::theme::{self, Tokens};

enum Runtime {
    NotStarted,
    /// No model is installed yet: the setup screen is showing.
    NeedsModel,
    Loading(Receiver<Result<Launched, String>>, Tier),
    Ready {
        server: Arc<LlamaServer>,
        tier: Tier,
    },
    Unavailable(String),
}

#[derive(Clone)]
enum Line {
    User(String),
    Assistant(String),
    Tool(String),
    Note(String),
}

struct Active {
    run: Run,
    start: Arc<lattice_engine::model::Workbook>,
    label: String,
    prompt: String,
    /// A tool ran (the workbook may have changed), so a crashed request must not be re-asked.
    tools_ran: bool,
}

/// A model download (or import) in progress.
struct Job {
    tier: Tier,
    progress: Arc<Progress>,
    rx: Receiver<Result<PathBuf, InstallError>>,
    /// When it started and how many bytes were already on disk then (for the time estimate).
    started: Option<(std::time::Instant, u64)>,
}

pub struct AssistantPane {
    pub open: bool,
    input: String,
    log: Vec<Line>,
    live: String,
    runtime: Runtime,
    harness: Option<Harness>,
    active: Option<Active>,
    queued: Option<String>,
    confirm: Option<(String, Json)>,
    /// The setup screen is open on request ("Manage models"), even though a model is running.
    manage: bool,
    jobs: Vec<Job>,
    /// The last download error per tier, shown on its card until the next attempt.
    errors: Vec<(Tier, String)>,
    recommendation: Option<Recommendation>,
    /// Automatic runtime restarts used for the current request (one is allowed).
    restarts: u32,
}

impl Default for AssistantPane {
    fn default() -> Self {
        AssistantPane {
            open: false,
            input: String::new(),
            log: Vec::new(),
            live: String::new(),
            runtime: Runtime::NotStarted,
            harness: None,
            active: None,
            queued: None,
            confirm: None,
            manage: false,
            jobs: Vec::new(),
            errors: Vec::new(),
            recommendation: None,
            restarts: 0,
        }
    }
}

impl AssistantPane {
    /// Start the sidecar for the best installed model, or show the setup screen when there is none.
    fn start_runtime(&mut self) {
        if !matches!(self.runtime, Runtime::NotStarted) {
            return;
        }
        let Some(server) = locate_server() else {
            self.runtime = Runtime::Unavailable(MISSING_RUNTIME.into());
            return;
        };
        let Some((tier, model)) = models::choose() else {
            self.runtime = Runtime::NeedsModel;
            return;
        };
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let _ = tx.send(launch(&server, &model, tier, pid_file()));
        });
        self.runtime = Runtime::Loading(rx, tier);
    }

    /// Stop the running model (if any) so the next request starts the current choice.
    fn restart_runtime(&mut self) {
        if let Runtime::Ready { server, .. } = &self.runtime {
            server.stop();
        }
        self.runtime = Runtime::NotStarted;
    }

    pub fn is_busy(&self) -> bool {
        self.active.is_some()
    }

    fn running_tier(&self) -> Option<Tier> {
        match &self.runtime {
            Runtime::Ready { tier, .. } | Runtime::Loading(_, tier) => Some(*tier),
            _ => None,
        }
    }

    fn job(&self, tier: Tier) -> Option<&Job> {
        self.jobs.iter().find(|j| j.tier == tier)
    }

    fn error(&self, tier: Tier) -> Option<&str> {
        self.errors.iter().find(|(t, _)| *t == tier).map(|(_, e)| e.as_str())
    }

    fn recommendation(&mut self) -> Recommendation {
        self.recommendation.get_or_insert_with(models::recommend).clone()
    }

    fn start_job(&mut self, tier: Tier, work: impl FnOnce(&Progress) -> Result<PathBuf, InstallError> + Send + 'static) {
        self.errors.retain(|(t, _)| *t != tier);
        let progress = Arc::new(Progress::default());
        let (tx, rx) = channel();
        let p = Arc::clone(&progress);
        std::thread::spawn(move || {
            let _ = tx.send(work(&p));
        });
        self.jobs.push(Job { tier, progress, rx, started: None });
    }
}

fn parse_tier(p: &Json) -> Result<Tier, String> {
    let s = p.get("tier").and_then(Json::as_str).unwrap_or("");
    Tier::parse(s).ok_or_else(|| format!("`tier` must be \"e4b\" or \"e2b\" (got {s:?})"))
}

fn models_dir() -> Result<PathBuf, String> {
    models::models_dir().ok_or_else(|| "this system has no folder for the assistant's models".to_string())
}

/// `assistant.models`: what is installed, downloading and recommended.
pub fn models_status(app: &mut SheetApp) -> Result<Json, String> {
    let pane = &mut app.assistant;
    let rec = pane.recommendation();
    let list: Vec<Json> = models::MODELS
        .iter()
        .map(|m| {
            let job = pane.job(m.tier);
            json!({
                "tier": m.tier.id(),
                "name": m.name,
                "bytes": m.bytes,
                "installed": models::installed(m.tier).is_some(),
                "recommended": rec.tier == m.tier,
                "downloading": job.is_some(),
                "done": job.map(|j| j.progress.done.load(Ordering::Relaxed)),
                "verifying": job.map(|j| j.progress.phase.load(Ordering::Relaxed) == PHASE_VERIFYING),
                "error": pane.error(m.tier),
                "downloadable": !models::urls(m.tier).is_empty(),
            })
        })
        .collect();
    let state = match &pane.runtime {
        Runtime::NotStarted => "notStarted",
        Runtime::NeedsModel => "needsModel",
        Runtime::Loading(..) => "loading",
        Runtime::Ready { .. } => "ready",
        Runtime::Unavailable(_) => "unavailable",
    };
    Ok(json!({
        "models": list,
        "recommended": rec.tier.id(),
        "reason": rec.reason,
        "active": pane.running_tier().map(Tier::id),
        "runtime": state,
        "runtimeBundled": locate_server().is_some(),
    }))
}

/// `assistant.install {tier: "e4b"|"e2b"|"both"}`: start downloading (already-installed models are skipped).
pub fn install(app: &mut SheetApp, p: &Json) -> Result<Json, String> {
    let tiers = if p.get("tier").and_then(Json::as_str).map(str::to_ascii_lowercase).as_deref() == Some("both") {
        vec![app.assistant.recommendation().tier, app.assistant.recommendation().tier.other()]
    } else {
        vec![parse_tier(p)?]
    };
    let dir = models_dir()?;
    let mut started = Vec::new();
    for tier in tiers {
        if app.assistant.job(tier).is_some() || models::installed(tier).is_some() {
            continue;
        }
        let target = Target::for_model(models::info(tier), models::urls(tier));
        let d = dir.clone();
        app.assistant.start_job(tier, move |progress| download::install(&target, &d, progress));
        started.push(tier.id());
    }
    app.assistant.open = true;
    Ok(json!({"started": started}))
}

/// `assistant.importModel {tier, path}`: verify a model file the user already has and copy it in (offline setup).
pub fn import_model(app: &mut SheetApp, p: &Json) -> Result<Json, String> {
    let tier = parse_tier(p)?;
    let path = p.get("path").and_then(Json::as_str).filter(|s| !s.trim().is_empty()).ok_or("missing `path`")?;
    if app.assistant.job(tier).is_some() {
        return Err("that model is already being installed".into());
    }
    let dir = models_dir()?;
    let target = Target::for_model(models::info(tier), Vec::new());
    let source = PathBuf::from(path);
    app.assistant.start_job(tier, move |progress| download::import(&target, &source, &dir, progress));
    Ok(json!({"started": [tier.id()]}))
}

/// `assistant.cancelInstall {tier?}`: stop a download (all of them without `tier`); the partial file is kept, so
/// a later Download resumes.
pub fn cancel_install(app: &mut SheetApp, p: &Json) -> Result<Json, String> {
    let only = if p.get("tier").is_some() { Some(parse_tier(p)?) } else { None };
    let mut n = 0;
    for j in &app.assistant.jobs {
        if only.is_none_or(|t| t == j.tier) {
            j.progress.cancel.store(true, Ordering::Relaxed);
            n += 1;
        }
    }
    Ok(json!({"cancelled": n}))
}

/// `assistant.removeModel {tier}`: delete a downloaded model (and its partial download) to free disk space.
pub fn remove_model(app: &mut SheetApp, p: &Json) -> Result<Json, String> {
    let tier = parse_tier(p)?;
    if app.assistant.job(tier).is_some() {
        return Err("that model is still downloading; cancel it first".into());
    }
    let m = models::info(tier);
    let dir = models_dir()?;
    let file = dir.join(m.file);
    let part = dir.join(format!("{}.part", m.file));
    if !file.exists() && !part.exists() {
        return Err(if models::installed(tier).is_some() {
            "this model came with this copy of Lattice and can't be removed here".into()
        } else {
            "that model isn't installed".into()
        });
    }
    if app.assistant.running_tier() == Some(tier) {
        if app.assistant.is_busy() {
            return Err("the assistant is using this model right now; wait for it to finish".into());
        }
        app.assistant.restart_runtime();
    }
    for f in [&file, &part] {
        if f.exists() {
            std::fs::remove_file(f).map_err(|e| format!("can't delete {}: {e}", f.display()))?;
        }
    }
    if matches!(app.assistant.runtime, Runtime::NeedsModel | Runtime::Unavailable(_)) {
        app.assistant.runtime = Runtime::NotStarted;
    }
    Ok(json!({"removed": tier.id()}))
}

/// `assistant.useModel {tier}`: run this installed model from now on (remembered across launches).
pub fn use_model(app: &mut SheetApp, p: &Json) -> Result<Json, String> {
    let tier = parse_tier(p)?;
    if models::installed(tier).is_none() {
        return Err("that model isn't installed; download it first".into());
    }
    models::set_preferred(tier)?;
    if app.assistant.running_tier() != Some(tier) {
        if app.assistant.is_busy() {
            return Err("the assistant is working; switch models when it has finished".into());
        }
        app.assistant.restart_runtime();
        if app.assistant.open {
            app.assistant.start_runtime();
        }
    }
    app.assistant.manage = false;
    Ok(json!({"active": tier.id()}))
}

/// `assistant.manageModels {open?}`: show or hide the model setup screen in the pane.
pub fn manage(app: &mut SheetApp, p: &Json) -> Result<Json, String> {
    let open = p.get("open").and_then(Json::as_bool).unwrap_or(!app.assistant.manage);
    app.assistant.manage = open;
    if open {
        app.assistant.open = true;
    }
    Ok(json!({"open": open}))
}

/// `assistant.ask {prompt}`: open the pane and start (or queue) a request.
pub fn ask(app: &mut SheetApp, prompt: &str) -> Result<Json, String> {
    let prompt = prompt.trim();
    if prompt.is_empty() {
        return Err("missing `prompt`".into());
    }
    if app.assistant.is_busy() {
        return Err("the assistant is still working on the previous request".into());
    }
    app.assistant.open = true;
    app.assistant.restarts = 0;
    app.assistant.start_runtime();
    if matches!(app.assistant.runtime, Runtime::NeedsModel) {
        return Err("no assistant model is installed yet; download one in the Assistant pane (assistant.install)".into());
    }
    app.assistant.queued = Some(prompt.to_string());
    Ok(json!({"queued": true}))
}

fn begin(app: &mut SheetApp, prompt: String) {
    let Runtime::Ready { server, .. } = &app.assistant.runtime else { return };
    if !server.is_running() {
        // The sidecar died between requests (crash, killed, sleep): start it again and keep the request.
        app.assistant.runtime = Runtime::NotStarted;
        app.assistant.queued = Some(prompt);
        app.assistant.start_runtime();
        return;
    }
    let generator: Arc<dyn Generator> = server.clone();
    let harness = app.assistant.harness.get_or_insert_with(Harness::default);
    let Some(start) = app.session.active().map(|d| d.wb.clone()) else { return };
    let thinking = use_thinking(Entry::Chat, &prompt);
    let run = with_session(&mut app.session, |host| Run::start(generator, harness, host, &prompt, thinking));
    let label = format!("Assistant: {}", prompt.chars().take(40).collect::<String>());
    app.assistant.log.push(Line::User(prompt.clone()));
    app.assistant.live.clear();
    app.assistant.active = Some(Active { run, start, label, prompt, tools_ran: false });
}

/// Finished downloads: report errors, and start the runtime when the first model arrives.
fn tick_jobs(app: &mut SheetApp, ctx: &egui::Context) {
    if app.assistant.jobs.is_empty() {
        return;
    }
    let mut finished = Vec::new();
    for (i, job) in app.assistant.jobs.iter_mut().enumerate() {
        if job.started.is_none() && job.progress.phase.load(Ordering::Relaxed) != PHASE_CONNECTING {
            #[allow(clippy::disallowed_methods)] // desktop only (the pane isn't built for wasm)
            let now = std::time::Instant::now();
            job.started = Some((now, job.progress.done.load(Ordering::Relaxed)));
        }
        match job.rx.try_recv() {
            Ok(r) => finished.push((i, r)),
            Err(TryRecvError::Disconnected) => finished.push((i, Err(InstallError::Failed("the download stopped unexpectedly".into())))),
            Err(TryRecvError::Empty) => {}
        }
    }
    for (i, r) in finished.into_iter().rev() {
        let tier = app.assistant.jobs.remove(i).tier;
        match r {
            Ok(_) => {
                if models::preferred().is_none() {
                    let _ = models::set_preferred(tier);
                }
                if matches!(app.assistant.runtime, Runtime::NeedsModel | Runtime::Unavailable(_)) {
                    app.assistant.runtime = Runtime::NotStarted;
                    app.assistant.manage = false;
                    app.assistant.start_runtime();
                }
            }
            Err(InstallError::Cancelled) => {}
            Err(InstallError::Failed(e)) => {
                log::warn!("assistant: {} download failed: {e}", tier.id());
                app.assistant.errors.push((tier, e));
            }
        }
    }
    ctx.request_repaint_after(Duration::from_millis(250));
}

/// Advance the runtime and the active request; called every frame while the pane exists.
fn tick(app: &mut SheetApp, ctx: &egui::Context) {
    tick_jobs(app, ctx);
    if let Runtime::Loading(rx, tier) = &app.assistant.runtime {
        let tier = *tier;
        match rx.try_recv() {
            Ok(Ok(l)) => {
                for n in &l.notes {
                    app.assistant.log.push(Line::Note(n.clone()));
                }
                app.assistant.runtime = Runtime::Ready { server: Arc::new(l.server), tier };
            }
            Ok(Err(e)) => app.assistant.runtime = Runtime::Unavailable(e),
            Err(TryRecvError::Disconnected) => app.assistant.runtime = Runtime::Unavailable("the assistant failed to start".into()),
            Err(TryRecvError::Empty) => ctx.request_repaint_after(Duration::from_millis(200)),
        }
    }
    if matches!(app.assistant.runtime, Runtime::Ready { .. })
        && app.assistant.active.is_none()
        && let Some(p) = app.assistant.queued.take()
    {
        begin(app, p);
    }
    if let Runtime::Unavailable(e) = &app.assistant.runtime
        && app.assistant.queued.take().is_some()
    {
        let e = e.clone();
        app.assistant.log.push(Line::Note(e));
    }
    let Some(mut active) = app.assistant.active.take() else { return };
    let harness = app.assistant.harness.get_or_insert_with(Harness::default);
    let events = with_session(&mut app.session, |host| active.run.poll(harness, host));
    let mut changed = false;
    let mut runtime_died = false;
    for ev in events {
        match ev {
            Event::Text(t) => app.assistant.live.push_str(&t),
            Event::ToolStarted { .. } => {}
            Event::ToolFinished { name, observation } => {
                changed = true;
                active.tools_ran = true;
                app.assistant.live.clear();
                let first = observation.lines().next().unwrap_or("").chars().take(90).collect::<String>();
                app.assistant.log.push(Line::Tool(format!("{name}: {first}")));
            }
            Event::NeedsConfirmation { name, args } => app.assistant.confirm = Some((name, args)),
            Event::Answer(a) => app.assistant.log.push(Line::Assistant(a)),
            Event::Warning(w) => app.assistant.log.push(Line::Note(w)),
            Event::Failed(e) => {
                let dead = matches!(&app.assistant.runtime, Runtime::Ready { server, .. } if !server.is_running());
                if dead && app.assistant.restarts == 0 && !active.tools_ran {
                    runtime_died = true;
                } else {
                    app.assistant.log.push(Line::Note(e));
                }
            }
        }
    }
    if changed {
        app.after_engine();
    }
    if active.run.is_finished() {
        if let Some(d) = app.session.active_mut() {
            d.squash_history_since(&active.start, &active.label);
        }
        app.assistant.live.clear();
        app.assistant.confirm = None;
        if runtime_died {
            // The model process crashed mid-request: restart it once and re-ask (nothing was changed yet).
            app.assistant.restarts += 1;
            let _ = app.assistant.log.pop_if(|l| matches!(l, Line::User(_)));
            app.assistant.log.push(Line::Note("The assistant's model stopped unexpectedly; restarting it…".into()));
            app.assistant.runtime = Runtime::NotStarted;
            app.assistant.queued = Some(active.prompt);
            app.assistant.start_runtime();
        }
    } else {
        ctx.request_repaint_after(Duration::from_millis(60));
        app.assistant.active = Some(active);
    }
}

fn live_status(live: &str) -> String {
    if live.contains("<|channel>") && !live.contains("<channel|>") {
        "Thinking…".into()
    } else if live.contains("<|tool_call>") {
        "Using tools…".into()
    } else if live.trim().is_empty() {
        "Working…".into()
    } else {
        live.rsplit("<channel|>").next().unwrap_or(live).trim().to_string()
    }
}

/// Run a pane command and show its error in the pane.
fn act(app: &mut SheetApp, id: &str, params: Json) {
    if let Err(e) = app.run(id, params) {
        app.assistant.log.push(Line::Note(e));
    }
}

/// "12 min left"-style estimate from the bytes downloaded since the job started.
fn eta(job: &Job) -> Option<String> {
    let (t0, b0) = job.started?;
    let done = job.progress.done.load(Ordering::Relaxed);
    let total = job.progress.total.load(Ordering::Relaxed);
    let secs = t0.elapsed().as_secs_f64();
    let rate = done.saturating_sub(b0) as f64 / secs.max(0.001);
    if secs < 3.0 || rate < 1.0 || total <= done {
        return None;
    }
    let left = (total - done) as f64 / rate;
    Some(if left < 90.0 { "under a minute left".into() } else { format!("about {} min left", (left / 60.0).round() as u64) })
}

fn progress_text(job: &Job) -> String {
    let p = &job.progress;
    let (done, total) = (p.done.load(Ordering::Relaxed), p.total.load(Ordering::Relaxed));
    match p.phase.load(Ordering::Relaxed) {
        PHASE_CONNECTING => "Connecting…".into(),
        PHASE_VERIFYING => format!("Checking the file… {:.0}%", p.fraction() * 100.0),
        _ => {
            let base = format!("{} of {}", human_bytes(done), human_bytes(total));
            match eta(job) {
                Some(e) => format!("{base} · {e}"),
                None => base,
            }
        }
    }
}

/// One model's card on the setup screen.
fn model_card(app: &mut SheetApp, ui: &mut Ui, t: &Tokens, m: &ModelInfo, recommended: bool) {
    let installed = models::installed(m.tier);
    let downloadable = !models::urls(m.tier).is_empty();
    let border = if recommended { t.accent } else { t.separator };
    egui::Frame::NONE.fill(t.window).stroke(egui::Stroke::new(1.0, border)).corner_radius(6.0).inner_margin(10.0).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(RichText::new(m.name).font(theme::ui_bold(14.0)));
            if recommended {
                egui::Frame::NONE.fill(t.accent_soft).corner_radius(4.0).inner_margin(egui::Margin::symmetric(6, 2)).show(ui, |ui| {
                    ui.label(RichText::new("Best for this device").small().color(t.accent_dark));
                });
            }
        });
        ui.label(RichText::new(m.summary).font(theme::ui_font(12.5)));
        ui.label(RichText::new(format!("{} download · needs {} GB of memory", human_bytes(m.bytes), m.min_ram_gb)).small().color(t.text_dim));
        ui.add_space(4.0);
        for p in m.pros {
            ui.label(RichText::new(format!("+  {p}")).font(theme::ui_font(12.0)));
        }
        for c in m.cons {
            ui.label(RichText::new(format!("–  {c}")).font(theme::ui_font(12.0)).color(t.text_dim));
        }
        ui.add_space(6.0);
        let tier = json!({"tier": m.tier.id()});
        if let Some(job) = app.assistant.job(m.tier) {
            let (frac, text) = (job.progress.fraction(), progress_text(job));
            ui.add(egui::ProgressBar::new(frac).text(text).desired_height(16.0));
            if ui.button("Cancel").clicked() {
                act(app, "assistant.cancelInstall", tier);
            }
            return;
        }
        if let Some(e) = app.assistant.error(m.tier) {
            ui.label(RichText::new(e).font(theme::ui_font(12.0)).color(t.danger));
        }
        ui.horizontal(|ui| {
            if installed.is_some() {
                let active = app.assistant.running_tier() == Some(m.tier);
                ui.label(RichText::new(if active { "✔ Installed · in use" } else { "✔ Installed" }).color(t.text_dim));
                if !active && ui.button("Use this model").clicked() {
                    act(app, "assistant.useModel", tier.clone());
                }
                if ui.button("Remove").on_hover_text("Delete the downloaded file to free disk space").clicked() {
                    act(app, "assistant.removeModel", tier);
                }
            } else {
                let label = if app.assistant.error(m.tier).is_some() { "Try again" } else { "Download" };
                if ui.add_enabled(downloadable, egui::Button::new(label)).clicked() {
                    act(app, "assistant.install", tier.clone());
                }
                if ui.link("Use a file I already have…").clicked()
                    && let Some(pick) = &app.services.pick_model
                    && let Some(path) = pick()
                {
                    act(app, "assistant.importModel", json!({"tier": m.tier.id(), "path": path}));
                }
            }
        });
    });
}

/// The setup screen: choose, download and manage models.
fn setup_view(app: &mut SheetApp, ui: &mut Ui, t: &Tokens) {
    let rec = app.assistant.recommendation();
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.label(RichText::new("Set up the assistant").font(theme::ui_bold(14.0)));
        ui.label(
            RichText::new(
                "The assistant runs entirely on this computer, so it needs a model file. Choose one to download; you can add the other later.",
            )
            .font(theme::ui_font(12.5)),
        );
        ui.add_space(4.0);
        ui.label(RichText::new(&rec.reason).small().color(t.text_dim));
        ui.add_space(8.0);
        // The recommended model first.
        let order = [models::info(rec.tier), models::info(rec.tier.other())];
        for m in order {
            model_card(app, ui, t, m, m.tier == rec.tier);
            ui.add_space(8.0);
        }
        let none_yet = models::MODELS.iter().all(|m| models::installed(m.tier).is_none() && app.assistant.job(m.tier).is_none());
        if none_yet && models::MODELS.iter().all(|m| !models::urls(m.tier).is_empty()) && ui.button("Download both").clicked() {
            act(app, "assistant.install", json!({"tier": "both"}));
        }
        if models::MODELS.iter().any(|m| models::urls(m.tier).is_empty()) {
            ui.label(
                RichText::new("Downloads aren't available in this build. If you have a model file, use \"Use a file I already have…\".")
                    .small()
                    .color(t.text_dim),
            );
        }
        ui.add_space(6.0);
        if let Some(dir) = models::models_dir() {
            ui.label(RichText::new(format!("Models are saved in {}", dir.display())).small().color(t.text_dim));
        }
        ui.label(
            RichText::new("The models are fine-tuned from Google's Gemma 4; their licence is listed in Lattice's ATTRIBUTION.md.")
                .small()
                .color(t.text_dim),
        );
        if app.assistant.manage
            && models::MODELS.iter().any(|m| models::installed(m.tier).is_some())
            && ui.button("Back to the conversation").clicked()
        {
            act(app, "assistant.manageModels", json!({"open": false}));
        }
    });
}

pub fn show(app: &mut SheetApp, ui: &mut Ui) {
    let ctx = ui.ctx().clone();
    tick(app, &ctx);
    if !app.assistant.open {
        return;
    }
    app.assistant.start_runtime();
    let t = Tokens::get(&ctx);
    egui::Panel::right("assistant_pane").default_size(340.0).resizable(true).frame(egui::Frame::NONE.fill(t.ribbon).inner_margin(10)).show(
        ui,
        |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Assistant").font(theme::ui_bold(15.0)));
                let status = match &app.assistant.runtime {
                    Runtime::Ready { tier: Tier::E4B, .. } => "local · Standard",
                    Runtime::Ready { tier: Tier::E2B, .. } => "local · Compact",
                    Runtime::Loading(..) => "starting the model…",
                    Runtime::NotStarted | Runtime::NeedsModel => "",
                    Runtime::Unavailable(_) => "unavailable",
                };
                ui.label(RichText::new(status).small().color(t.text_dim));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if crate::widgets::icon_button(ui, crate::icons::Icon::Close, t.text_dim, "Close", vec2(22.0, 22.0)).clicked() {
                        app.assistant.open = false;
                    }
                    if !matches!(app.assistant.runtime, Runtime::NeedsModel)
                        && ui.small_button("Models").on_hover_text("Download, switch or remove models").clicked()
                    {
                        act(app, "assistant.manageModels", json!({}));
                    }
                });
            });
            ui.separator();
            if matches!(app.assistant.runtime, Runtime::NeedsModel) || app.assistant.manage {
                setup_view(app, ui, &t);
                return;
            }
            if let Runtime::Unavailable(e) = &app.assistant.runtime {
                let e = e.clone();
                ui.label(RichText::new(e).color(t.text_dim));
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if locate_server().is_some() && ui.button("Try again").clicked() {
                        app.assistant.runtime = Runtime::NotStarted;
                    }
                    if ui.button("Manage models").clicked() {
                        act(app, "assistant.manageModels", json!({"open": true}));
                    }
                });
                return;
            }
            chat_view(app, ui, &t);
        },
    );
}

fn chat_view(app: &mut SheetApp, ui: &mut Ui, t: &Tokens) {
    let input_h = 92.0;
    egui::ScrollArea::vertical().max_height((ui.available_height() - input_h).max(80.0)).stick_to_bottom(true).show(ui, |ui| {
        ui.set_width(ui.available_width());
        if matches!(app.assistant.runtime, Runtime::Loading(..)) && app.assistant.log.is_empty() {
            ui.label(RichText::new("Starting the model (the first start can take a minute)…").italics().color(t.text_dim));
        }
        for line in &app.assistant.log {
            match line {
                Line::User(s) => {
                    egui::Frame::NONE.fill(t.window).corner_radius(6.0).inner_margin(6.0).show(ui, |ui| {
                        ui.label(RichText::new(s).font(theme::ui_font(13.0)));
                    });
                }
                Line::Assistant(s) => {
                    ui.label(RichText::new(s).font(theme::ui_font(13.0)));
                }
                Line::Tool(s) => {
                    ui.label(RichText::new(format!("▸ {s}")).small().color(t.text_dim));
                }
                Line::Note(s) => {
                    ui.label(RichText::new(s).italics().color(t.text_dim));
                }
            }
            ui.add_space(6.0);
        }
        if app.assistant.active.is_some() {
            ui.label(RichText::new(live_status(&app.assistant.live)).italics().color(t.text_dim));
        } else if app.assistant.queued.is_some() {
            ui.label(RichText::new("Waiting for the model to start…").italics().color(t.text_dim));
        }
    });
    if let Some((name, args)) = app.assistant.confirm.clone() {
        let what = args.get("command").and_then(Json::as_str).unwrap_or(&name).to_string();
        egui::Frame::NONE.fill(t.window).corner_radius(6.0).inner_margin(6.0).show(ui, |ui| {
            ui.label(format!("The assistant wants to run {what}."));
            ui.horizontal(|ui| {
                let mut answer = None;
                if ui.button("Allow").clicked() {
                    answer = Some(true);
                }
                if ui.button("Don't allow").clicked() {
                    answer = Some(false);
                }
                if let (Some(yes), Some(active)) = (answer, app.assistant.active.as_mut()) {
                    active.run.confirm(yes);
                    app.assistant.confirm = None;
                }
            });
        });
    }
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
        ui.label(RichText::new("Runs on this device. Undo reverses a whole request.").small().color(t.text_dim));
        ui.horizontal(|ui| {
            let busy = app.assistant.is_busy();
            let edit = ui.add_sized(
                vec2(ui.available_width() - 64.0, 48.0),
                egui::TextEdit::multiline(&mut app.assistant.input).hint_text("Ask about this workbook, or tell me what to change").desired_rows(2),
            );
            let enter = edit.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);
            if busy {
                if ui.button("Stop").clicked()
                    && let Some(active) = app.assistant.active.as_mut()
                {
                    active.run.cancel();
                }
            } else if (ui.button("Send").clicked() || enter) && !app.assistant.input.trim().is_empty() {
                let p = std::mem::take(&mut app.assistant.input);
                if let Err(e) = ask(app, &p) {
                    app.assistant.log.push(Line::Note(e));
                }
            }
        });
    });
}
