//! One assistant request as a pollable state machine.
//!
//! The model generates on a worker thread (seconds to minutes on a laptop CPU), so the UI never blocks; tool calls
//! run on the caller's thread inside [`Run::poll`], where the host owns the workbook session. Risky calls pause the
//! run until the user answers ([`Run::confirm`]).

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, channel};

use serde_json::{Value, json};

use crate::chat::render;
use crate::harness::{Harness, ToolHost, is_risky};
use crate::parse::{ToolCall, parse_tool_calls, split_thinking};
use crate::prompt::{system_message, user_message};
use crate::runtime::{GenOutput, GenParams, Generator};

/// Model turns per request before the run stops (each turn may hold several tool calls).
pub const MAX_STEPS: usize = 10;
pub const MAX_CALLS_PER_TURN: usize = 8;

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    /// Streamed model text (answer or thought) for live display.
    Text(String),
    ToolStarted {
        name: String,
        args: Value,
    },
    ToolFinished {
        name: String,
        observation: String,
    },
    /// A risky call is waiting for the user; answer with [`Run::confirm`].
    NeedsConfirmation {
        name: String,
        args: Value,
    },
    Answer(String),
    /// Shown with the answer: the model's claim and what actually happened disagree (e.g. every action failed).
    Warning(String),
    Failed(String),
}

/// The same tool result this many times in a row means the model is stuck; the run stops early.
pub const MAX_REPEATS: usize = 3;

enum Worker {
    Text(String),
    Done(Result<GenOutput, String>),
}

enum State {
    Generating(Receiver<Worker>),
    Executing,
    AwaitingConfirmation,
    Finished,
}

pub struct Run {
    generator: Arc<dyn Generator>,
    pub messages: Vec<Value>,
    pub thinking: bool,
    state: State,
    stop: Arc<AtomicBool>,
    pending: VecDeque<ToolCall>,
    decision: Option<bool>,
    next_call_id: usize,
    steps: usize,
    pub tokens_in: u64,
    pub tokens_out: u64,
    /// Tool calls that succeeded / failed (observation `ERROR:`) in this request.
    ok_calls: usize,
    failed_calls: usize,
    /// The last observation and how many times in a row it came back.
    last_observation: (String, usize),
}

impl Run {
    /// Start a request: snapshot the workbook, build the prompt and begin generating.
    pub fn start(generator: Arc<dyn Generator>, harness: &Harness, host: &mut dyn ToolHost, request: &str, thinking: bool) -> Run {
        let messages = vec![system_message(), user_message(&harness.snapshot(host), request)];
        let mut run = Run {
            generator,
            messages,
            thinking,
            state: State::Finished,
            stop: Arc::new(AtomicBool::new(false)),
            pending: VecDeque::new(),
            decision: None,
            next_call_id: 0,
            steps: 0,
            tokens_in: 0,
            tokens_out: 0,
            ok_calls: 0,
            failed_calls: 0,
            last_observation: (String::new(), 0),
        };
        run.generate();
        run
    }

    fn params(&self) -> GenParams {
        GenParams { max_tokens: if self.thinking { 4096 } else { 1536 }, ..GenParams::default() }
    }

    fn generate(&mut self) {
        self.steps += 1;
        let prompt = render(&self.messages, self.thinking, true);
        let (tx, rx) = channel();
        let generator = Arc::clone(&self.generator);
        let stop = Arc::clone(&self.stop);
        let params = self.params();
        std::thread::spawn(move || {
            let r = generator.generate(&prompt, &params, &mut |t| {
                let _ = tx.send(Worker::Text(t.to_string()));
                !stop.load(Ordering::Relaxed)
            });
            let _ = tx.send(Worker::Done(r));
        });
        self.state = State::Generating(rx);
    }

    pub fn is_finished(&self) -> bool {
        matches!(self.state, State::Finished)
    }

    pub fn is_waiting_for_user(&self) -> bool {
        matches!(self.state, State::AwaitingConfirmation)
    }

    /// Stop after the current step; generation is cut short.
    pub fn cancel(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        self.pending.clear();
        self.state = State::Finished;
    }

    /// The user's answer to [`Event::NeedsConfirmation`].
    pub fn confirm(&mut self, yes: bool) {
        if matches!(self.state, State::AwaitingConfirmation) {
            self.decision = Some(yes);
            self.state = State::Executing;
        }
    }

    /// Advance the run; call every frame. Tool calls run here, against `host`.
    pub fn poll(&mut self, harness: &Harness, host: &mut dyn ToolHost) -> Vec<Event> {
        let mut events = Vec::new();
        if let State::Generating(rx) = &self.state {
            let mut done = None;
            loop {
                match rx.try_recv() {
                    Ok(Worker::Text(t)) => events.push(Event::Text(t)),
                    Ok(Worker::Done(r)) => {
                        done = Some(r);
                        break;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        done = Some(Err("the assistant runtime stopped unexpectedly".into()));
                        break;
                    }
                }
            }
            match done {
                None => return events,
                Some(Err(e)) => {
                    self.state = State::Finished;
                    events.push(Event::Failed(e));
                    return events;
                }
                Some(Ok(out)) => self.on_generated(out, &mut events),
            }
        }
        if matches!(self.state, State::Executing) {
            self.execute(harness, host, &mut events);
        }
        events
    }

    fn on_generated(&mut self, out: GenOutput, events: &mut Vec<Event>) {
        self.tokens_in += out.prompt_tokens;
        self.tokens_out += out.generated_tokens;
        let (thought, after, closed) = if self.thinking { split_thinking(&out.text) } else { (String::new(), out.text.clone(), true) };
        let calls = if closed { parse_tool_calls(if self.thinking { &after } else { &out.text }) } else { Vec::new() };
        if calls.is_empty() || self.stop.load(Ordering::Relaxed) {
            let (_, fin, _) = split_thinking(&out.text);
            let mut msg = json!({"role": "assistant", "content": fin});
            if !thought.is_empty() {
                msg["reasoning_content"] = Value::String(thought);
            }
            self.messages.push(msg);
            self.state = State::Finished;
            events.push(if fin.is_empty() && out.truncated {
                Event::Failed("the answer was cut off; try a narrower request".into())
            } else {
                Event::Answer(if fin.is_empty() { "Done.".into() } else { fin })
            });
            if self.failed_calls > 0 && self.ok_calls == 0 {
                events.push(Event::Warning(
                    "None of the assistant's actions worked, so the workbook wasn't changed. Try rephrasing the request.".into(),
                ));
            }
            return;
        }
        let calls: Vec<ToolCall> = calls.into_iter().take(MAX_CALLS_PER_TURN).collect();
        let tool_calls: Vec<Value> = calls
            .iter()
            .enumerate()
            .map(|(k, c)| json!({"id": format!("call_{}", self.next_call_id + k), "type": "function", "function": {"name": c.name, "arguments": c.arguments}}))
            .collect();
        let mut msg = json!({"role": "assistant", "content": "", "tool_calls": tool_calls});
        if !thought.is_empty() {
            msg["reasoning_content"] = Value::String(thought);
        }
        self.messages.push(msg);
        self.pending = calls.into();
        self.state = State::Executing;
    }

    fn execute(&mut self, harness: &Harness, host: &mut dyn ToolHost, events: &mut Vec<Event>) {
        while let Some(call) = self.pending.front().cloned() {
            if is_risky(&call.name, &call.arguments) && self.decision.is_none() {
                self.state = State::AwaitingConfirmation;
                events.push(Event::NeedsConfirmation { name: call.name, args: call.arguments });
                return;
            }
            let decision = self.decision.take().unwrap_or(true);
            self.pending.pop_front();
            events.push(Event::ToolStarted { name: call.name.clone(), args: call.arguments.clone() });
            let observation = harness.run(host, &call.name, &call.arguments, &mut |_, _| decision);
            self.messages
                .push(json!({"role": "tool", "tool_call_id": format!("call_{}", self.next_call_id), "name": call.name, "content": observation}));
            self.next_call_id += 1;
            if observation.starts_with("ERROR") {
                self.failed_calls += 1;
            } else {
                self.ok_calls += 1;
            }
            let key = format!("{}\n{observation}", call.name);
            self.last_observation = if self.last_observation.0 == key { (key, self.last_observation.1 + 1) } else { (key, 1) };
            events.push(Event::ToolFinished { name: call.name, observation });
            if self.last_observation.1 >= MAX_REPEATS {
                self.pending.clear();
                self.state = State::Finished;
                events.push(Event::Failed(
                    "The assistant kept getting the same result without making progress, so it stopped. Undo removes its changes.".into(),
                ));
                return;
            }
        }
        if self.steps >= MAX_STEPS {
            self.state = State::Finished;
            events.push(Event::Failed(format!("stopped after {MAX_STEPS} steps; the changes so far are kept (Undo removes them)")));
            return;
        }
        self.generate();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Replays scripted model outputs.
    struct Script(Mutex<VecDeque<String>>);

    impl Generator for Script {
        fn generate(&self, _: &str, _: &GenParams, on_text: &mut dyn FnMut(&str) -> bool) -> Result<GenOutput, String> {
            let text = self.0.lock().map_err(|e| e.to_string())?.pop_front().ok_or("script exhausted")?;
            on_text(&text);
            Ok(GenOutput { text, truncated: false, prompt_tokens: 10, generated_tokens: 5 })
        }
    }

    fn run_to_end(run: &mut Run, h: &Harness, host: &mut dyn ToolHost, confirm: Option<bool>) -> Vec<Event> {
        let mut all = Vec::new();
        for _ in 0..2000 {
            let ev = run.poll(h, host);
            if ev.iter().any(|e| matches!(e, Event::NeedsConfirmation { .. })) {
                run.confirm(confirm.unwrap_or(false));
            }
            all.extend(ev);
            if run.is_finished() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        all
    }

    #[test]
    fn runs_tool_calls_and_answers() {
        let script = Script(Mutex::new(VecDeque::from([
            "<|tool_call>call:set_cell{cell:<|\"|>D2<|\"|>,input:<|\"|>=B2*2<|\"|>}<tool_call|>".to_string(),
            "Put =B2*2 in D2.".to_string(),
        ])));
        let mut host = lattice_mcp::Headless::new();
        let h = Harness::default();
        let mut run = Run::start(Arc::new(script), &h, &mut host, "Double B2 into D2", false);
        let ev = run_to_end(&mut run, &h, &mut host, None);
        assert!(ev.contains(&Event::Answer("Put =B2*2 in D2.".into())), "{ev:?}");
        assert!(ev.iter().any(|e| matches!(e, Event::ToolFinished { observation, .. } if observation.starts_with("OK"))));
        assert_eq!(host.session.execute("cell.get", json!({"cell": "D2"})).unwrap()["formula"], "=B2*2");
        assert_eq!(run.messages.len(), 5, "system, user, call, tool, answer");
    }

    #[test]
    fn claimed_success_after_only_failures_is_flagged() {
        // What E4B did in the smoke test: invented tool names, then "Bolded B4:I4".
        let script = Script(Mutex::new(VecDeque::from([
            "<|tool_call>call:format__bold{range:<|\"|>B4:I4<|\"|>}<tool_call|>".to_string(),
            "Bolded B4:I4.".to_string(),
        ])));
        let mut host = lattice_mcp::Headless::new();
        let h = Harness::default();
        let mut run = Run::start(Arc::new(script), &h, &mut host, "Bold the header", false);
        let ev = run_to_end(&mut run, &h, &mut host, None);
        assert!(ev.contains(&Event::Answer("Bolded B4:I4.".into())));
        assert!(matches!(ev.last(), Some(Event::Warning(w)) if w.contains("wasn't changed")), "{ev:?}");
        // One success among failures is not flagged.
        let mixed = Script(Mutex::new(VecDeque::from([
            "<|tool_call>call:format__bold{}<tool_call|><|tool_call>call:set_cell{cell:<|\"|>A1<|\"|>,input:<|\"|>1<|\"|>}<tool_call|>".to_string(),
            "Done.".to_string(),
        ])));
        let mut run = Run::start(Arc::new(mixed), &h, &mut host, "x", false);
        let ev = run_to_end(&mut run, &h, &mut host, None);
        assert!(!ev.iter().any(|e| matches!(e, Event::Warning(_))), "{ev:?}");
    }

    #[test]
    fn risky_calls_wait_for_the_user() {
        let script = Script(Mutex::new(VecDeque::from([
            "<|tool_call>call:execute_command{command:<|\"|>file.new<|\"|>}<tool_call|>".to_string(),
            "Okay, I left your workbook open.".to_string(),
        ])));
        let mut host = lattice_mcp::Headless::new();
        let h = Harness::default();
        let mut run = Run::start(Arc::new(script), &h, &mut host, "new workbook", false);
        let ev = run_to_end(&mut run, &h, &mut host, Some(false));
        assert!(ev.iter().any(|e| matches!(e, Event::NeedsConfirmation { .. })));
        assert!(ev.iter().any(|e| matches!(e, Event::ToolFinished { observation, .. } if observation.starts_with("DECLINED"))));
    }

    #[test]
    fn step_limit_and_runtime_errors_end_the_run() {
        let busy = Script(Mutex::new(
            (0..20).map(|i| format!("<|tool_call>call:set_cell{{cell:<|\"|>A{}<|\"|>,input:<|\"|>{i}<|\"|>}}<tool_call|>", i + 1)).collect(),
        ));
        let mut host = lattice_mcp::Headless::new();
        let h = Harness::default();
        let mut run = Run::start(Arc::new(busy), &h, &mut host, "busy", false);
        let ev = run_to_end(&mut run, &h, &mut host, None);
        assert!(matches!(ev.last(), Some(Event::Failed(m)) if m.contains("stopped after")), "{:?}", ev.last());
        let looping = Script(Mutex::new((0..20).map(|_| "<|tool_call>call:inspect_workbook{}<tool_call|>".to_string()).collect()));
        let mut run = Run::start(Arc::new(looping), &h, &mut host, "loop", false);
        let ev = run_to_end(&mut run, &h, &mut host, None);
        assert!(matches!(ev.last(), Some(Event::Failed(m)) if m.contains("same result")), "{:?}", ev.last());
        assert_eq!(ev.iter().filter(|e| matches!(e, Event::ToolFinished { .. })).count(), MAX_REPEATS);
        let empty = Script(Mutex::new(VecDeque::new()));
        let mut run = Run::start(Arc::new(empty), &h, &mut host, "x", false);
        let ev = run_to_end(&mut run, &h, &mut host, None);
        assert!(matches!(ev.last(), Some(Event::Failed(_))));
    }
}
