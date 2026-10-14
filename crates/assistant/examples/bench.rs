//! End-user harness benchmark. Runs each job through the same agent loop the app ships (`Run`, the prompt, the tool
//! parser, the thinking router and the llama-server sidecar), then writes one JSON line per job: the final answer,
//! how the run ended, the transcript and the saved workbook. Grading happens outside this binary
//! (eval_lattice/run_end_user.py), which reopens the saved workbook and reads the transcript.
//!
//! `cargo run --release -p lattice-assistant --example bench -- --server path/llama-server --model path/x.gguf
//!  --jobs jobs.jsonl --out results.jsonl [--threads 4] [--gpu-layers 99] [--resume]`
//!
//! Each job is one JSON line: `{"id", "request", "sample"?, "open"?, "save"?}`. `open` loads a workbook file before the
//! run (what the model sees), `save` writes the workbook after it. Risky tools (open/save/new, file.*, app.*) are
//! confirmed automatically, as the Python eval does, since nobody is there to click.

// A desktop-only timing harness: the clock is read off wasm (the assistant crate isn't built for wasm).
#![allow(clippy::disallowed_methods)]

use std::fs::{self, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::time::{Duration, Instant};

use lattice_assistant::agent::{Event, Run};
use lattice_assistant::install::free_port;
use lattice_assistant::router::{Entry, use_thinking};
use lattice_assistant::runtime::{Generator, LlamaServer, RuntimeConfig};
use lattice_assistant::{Harness, ToolHost};
use serde_json::{Value, json};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let server = get("--server").unwrap_or_else(|| "llama-server".into());
    let model = get("--model").unwrap_or_default();
    let jobs_path = get("--jobs").unwrap_or_default();
    let out_path = get("--out").unwrap_or_default();
    let threads = get("--threads").and_then(|t| t.parse().ok()).unwrap_or(4);
    let gpu_layers = get("--gpu-layers").and_then(|t| t.parse().ok()).unwrap_or(0);
    let resume = args.iter().any(|a| a == "--resume");
    if jobs_path.is_empty() || out_path.is_empty() || (model.is_empty() && !args.iter().any(|a| a == "--prompt-only")) {
        eprintln!(
            "usage: bench --model X.gguf --jobs jobs.jsonl --out results.jsonl [--server llama-server] [--threads N] [--gpu-layers N] [--resume]"
        );
        std::process::exit(2);
    }

    if args.iter().any(|a| a == "--prompt-only") {
        // The prompts the app would send for each job (system and user text), with no model involved. Other
        // harnesses (opencode) use them so every harness starts from the same workbook snapshot and instructions.
        let text = fs::read_to_string(&jobs_path).unwrap_or_default();
        let h = Harness::default();
        let mut file = BufWriter::new(fs::File::create(&out_path).unwrap_or_else(|e| {
            eprintln!("out: {e}");
            std::process::exit(1);
        }));
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let Ok(job) = serde_json::from_str::<Value>(line) else { continue };
            let mut host = lattice_mcp::Headless::new();
            setup_workbook(&job, &mut host);
            let request = job["request"].as_str().unwrap_or_default();
            let user = lattice_assistant::prompt::user_message(&h.snapshot(&mut host), request);
            let row = json!({"id": job["id"], "system": lattice_assistant::prompt::SYSTEM, "user": user["content"]});
            if writeln!(file, "{row}").is_err() {
                std::process::exit(1);
            }
        }
        let _ = file.flush();
        return;
    }

    // Jobs already in the output file are skipped on --resume (a long run can be stopped and continued).
    let mut done = std::collections::HashSet::new();
    if resume {
        if let Ok(text) = fs::read_to_string(&out_path) {
            for line in text.lines() {
                if let Ok(v) = serde_json::from_str::<Value>(line) {
                    if let Some(id) = v["id"].as_str() {
                        done.insert(id.to_string());
                    }
                }
            }
        }
    }

    let cfg = RuntimeConfig {
        server: server.into(),
        model: model.clone().into(),
        ctx_size: 16384,
        threads,
        gpu_layers,
        port: free_port().unwrap_or(18080),
    };
    let t0 = Instant::now();
    let srv = match LlamaServer::spawn(&cfg, Duration::from_secs(300)) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("runtime: {e}");
            std::process::exit(1);
        }
    };
    eprintln!("runtime ready in {:.1}s: {model}", t0.elapsed().as_secs_f64());

    let jobs = match fs::read_to_string(&jobs_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("jobs: {e}");
            std::process::exit(1);
        }
    };
    let file = match OpenOptions::new().create(true).append(true).open(&out_path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("out: {e}");
            std::process::exit(1);
        }
    };
    let mut out = BufWriter::new(file);
    let h = Harness::default();
    let generator: Arc<dyn Generator> = srv.clone();
    let lines: Vec<&str> = jobs.lines().filter(|l| !l.trim().is_empty()).collect();
    let mut ran = 0;
    for line in &lines {
        let job: Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("skipping a bad job line: {e}");
                continue;
            }
        };
        let id = job["id"].as_str().unwrap_or_default().to_string();
        if done.contains(&id) {
            continue;
        }
        let t = Instant::now();
        let result = run_job(&job, &generator, &h);
        let secs = t.elapsed().as_secs_f64();
        ran += 1;
        let end = result["end"].as_str().unwrap_or("?").to_string();
        let calls = result["calls"].as_u64().unwrap_or(0);
        eprintln!("[{}/{}] {id}: {end}, {calls} calls, {secs:.1}s", ran, lines.len() - done.len());
        if writeln!(out, "{result}").and_then(|_| out.flush()).is_err() {
            eprintln!("out: write failed");
            std::process::exit(1);
        }
    }
    srv.stop();
    eprintln!("done: {ran} jobs in {:.1}s", t0.elapsed().as_secs_f64());
}

/// Loads the job's starting workbook (a sample or a file). Returns the error text when that fails.
fn setup_workbook(job: &Value, host: &mut lattice_mcp::Headless) -> Option<String> {
    let mut setup_error = None;
    if let Some(sample) = job["sample"].as_str() {
        if let Err(e) = ToolHost::call(host, "new_workbook", &json!({"sample": sample})) {
            setup_error = Some(format!("new_workbook: {e}"));
        }
    }
    if let Some(path) = job["open"].as_str() {
        if let Err(e) = ToolHost::call(host, "open_workbook", &json!({"path": path})) {
            setup_error = Some(format!("open_workbook: {e}"));
        }
    }
    setup_error
}

fn run_job(job: &Value, generator: &Arc<dyn Generator>, h: &Harness) -> Value {
    let id = job["id"].as_str().unwrap_or_default();
    let request = job["request"].as_str().unwrap_or_default();
    let mut host = lattice_mcp::Headless::new();
    let setup_error = setup_workbook(job, &mut host);

    let think = use_thinking(Entry::Chat, request);
    let t = Instant::now();
    let mut answer: Option<String> = None;
    let mut failed: Option<String> = None;
    let mut warnings: Vec<String> = Vec::new();
    let mut run = Run::start(Arc::clone(generator), h, &mut host, request, think);
    loop {
        for ev in run.poll(h, &mut host) {
            match ev {
                Event::NeedsConfirmation { .. } => run.confirm(true),
                Event::Answer(a) => answer = Some(a),
                Event::Warning(w) => warnings.push(w),
                Event::Failed(e) => failed = Some(e),
                Event::Text(_) | Event::ToolStarted { .. } | Event::ToolFinished { .. } => {}
            }
        }
        if run.is_finished() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let seconds = t.elapsed().as_secs_f64();

    // Transcript without the system and user prompts (the grader reads only the tool calls and answers).
    let transcript: Vec<Value> = run.messages.iter().skip(2).cloned().collect();
    let calls = transcript.iter().map(|m| m["tool_calls"].as_array().map_or(0, Vec::len)).sum::<usize>();
    let turns = transcript.iter().filter(|m| m["role"] == "assistant").count();

    let mut save_error = None;
    if let Some(path) = job["save"].as_str() {
        if let Err(e) = ToolHost::call(&mut host, "save_workbook", &json!({"path": path})) {
            save_error = Some(e);
        }
    }

    let end = match (&answer, &failed) {
        (Some(_), _) => "answered",
        (None, Some(_)) => "failed",
        (None, None) => "no_answer",
    };
    json!({
        "id": id,
        "final": answer.unwrap_or_default(),
        "end": end,
        "error": failed,
        "setup_error": setup_error,
        "save_error": save_error,
        "warnings": warnings,
        "thinking": think,
        "turns": turns,
        "calls": calls,
        "messages": transcript,
        "tokens_in": run.tokens_in,
        "tokens_out": run.tokens_out,
        "seconds": seconds,
    })
}
