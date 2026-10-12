//! The Assistant task pane (desktop only): a conversation with the bundled local model, which works on the open
//! workbook through Lattice's own tools. Each request is one undo step; file, app and window actions and sheet
//! deletion ask first. The model runs in a sidecar process started on first use.

use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError, channel};
use std::time::Duration;

use egui::{RichText, Ui, vec2};
use lattice_assistant::agent::{Event, Run};
use lattice_assistant::router::{Entry, Tier, detect_tier, gpu_layers, use_thinking};
use lattice_assistant::runtime::{Generator, LlamaServer, RuntimeConfig};
use lattice_assistant::{Harness, with_session};
use serde_json::Value as Json;

use crate::SheetApp;
use crate::theme::{self, Tokens};

enum Runtime {
    NotStarted,
    Loading(Receiver<Result<(Arc<LlamaServer>, Tier), String>>),
    Ready(Arc<LlamaServer>, Tier),
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
        }
    }
}

impl AssistantPane {
    fn start_runtime(&mut self) {
        if !matches!(self.runtime, Runtime::NotStarted) {
            return;
        }
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let r = (|| {
                let (server, model, tier) = lattice_assistant::install::locate(detect_tier())?;
                let port = lattice_assistant::install::free_port().ok_or("no free local port for the assistant")?;
                let cfg = RuntimeConfig { server, model, ctx_size: 16384, threads: 0, gpu_layers: gpu_layers(), port };
                LlamaServer::spawn(&cfg, Duration::from_secs(180)).map(|s| (Arc::new(s), tier))
            })();
            let _ = tx.send(r);
        });
        self.runtime = Runtime::Loading(rx);
    }

    pub fn is_busy(&self) -> bool {
        self.active.is_some()
    }
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
    app.assistant.queued = Some(prompt.to_string());
    app.assistant.start_runtime();
    Ok(serde_json::json!({"queued": true}))
}

fn begin(app: &mut SheetApp, prompt: String) {
    let Runtime::Ready(server, _) = &app.assistant.runtime else { return };
    let generator: Arc<dyn Generator> = server.clone();
    let harness = app.assistant.harness.get_or_insert_with(Harness::default);
    let Some(start) = app.session.active().map(|d| d.wb.clone()) else { return };
    let thinking = use_thinking(Entry::Chat, &prompt);
    let run = with_session(&mut app.session, |host| Run::start(generator, harness, host, &prompt, thinking));
    let label = format!("Assistant: {}", prompt.chars().take(40).collect::<String>());
    app.assistant.log.push(Line::User(prompt));
    app.assistant.live.clear();
    app.assistant.active = Some(Active { run, start, label });
}

/// Advance the runtime and the active request; called every frame while the pane exists.
fn tick(app: &mut SheetApp, ctx: &egui::Context) {
    if let Runtime::Loading(rx) = &app.assistant.runtime {
        match rx.try_recv() {
            Ok(Ok((server, tier))) => app.assistant.runtime = Runtime::Ready(server, tier),
            Ok(Err(e)) => app.assistant.runtime = Runtime::Unavailable(e),
            Err(TryRecvError::Disconnected) => app.assistant.runtime = Runtime::Unavailable("the assistant failed to start".into()),
            Err(TryRecvError::Empty) => ctx.request_repaint_after(Duration::from_millis(200)),
        }
    }
    if matches!(app.assistant.runtime, Runtime::Ready(..))
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
    for ev in events {
        match ev {
            Event::Text(t) => app.assistant.live.push_str(&t),
            Event::ToolStarted { .. } => {}
            Event::ToolFinished { name, observation } => {
                changed = true;
                app.assistant.live.clear();
                let first = observation.lines().next().unwrap_or("").chars().take(90).collect::<String>();
                app.assistant.log.push(Line::Tool(format!("{name}: {first}")));
            }
            Event::NeedsConfirmation { name, args } => app.assistant.confirm = Some((name, args)),
            Event::Answer(a) => app.assistant.log.push(Line::Assistant(a)),
            Event::Failed(e) => app.assistant.log.push(Line::Note(e)),
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
                    Runtime::Ready(_, Tier::E4B) => "local · E4B",
                    Runtime::Ready(_, Tier::E2B) => "local · E2B",
                    Runtime::Loading(_) => "loading model…",
                    Runtime::NotStarted => "",
                    Runtime::Unavailable(_) => "unavailable",
                };
                ui.label(RichText::new(status).small().color(t.text_dim));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if crate::widgets::icon_button(ui, crate::icons::Icon::Close, t.text_dim, "Close", vec2(22.0, 22.0)).clicked() {
                        app.assistant.open = false;
                    }
                });
            });
            ui.separator();
            if let Runtime::Unavailable(e) = &app.assistant.runtime {
                ui.label(RichText::new(e).color(t.text_dim));
                return;
            }
            let input_h = 92.0;
            egui::ScrollArea::vertical().max_height((ui.available_height() - input_h).max(80.0)).stick_to_bottom(true).show(ui, |ui| {
                ui.set_width(ui.available_width());
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
                        egui::TextEdit::multiline(&mut app.assistant.input)
                            .hint_text("Ask about this workbook, or tell me what to change")
                            .desired_rows(2),
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
        },
    );
}
