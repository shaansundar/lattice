//! Headless end-to-end assistant run: a real request against a sample workbook through the runtime sidecar,
//! the agent loop and Lattice's tools, with timings (used for the runtime benchmark).
//!
//! `cargo run --release -p lattice-assistant --example ask -- --server path/llama-server --model path/x.gguf
//!  [--sample sales] [--threads 4] [--gpu-layers 0] [--think] [--dump-prompt] "Bold the header row"`

// A desktop-only timing harness: the clock is read off wasm (the assistant crate isn't built for wasm).
#![allow(clippy::disallowed_methods)]

use std::sync::Arc;
use std::time::{Duration, Instant};

use lattice_assistant::Harness;
use lattice_assistant::agent::{Event, Run};
use lattice_assistant::chat::render;
use lattice_assistant::prompt::{system_message, user_message};
use lattice_assistant::runtime::{Generator, LlamaServer, RuntimeConfig};
use serde_json::json;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let server = get("--server").unwrap_or_else(|| "llama-server".into());
    let model = get("--model").unwrap_or_default();
    let sample = get("--sample").unwrap_or_else(|| "sales".into());
    let threads = get("--threads").and_then(|t| t.parse().ok()).unwrap_or(4);
    let gpu_layers = get("--gpu-layers").and_then(|t| t.parse().ok()).unwrap_or(0);
    let think = args.iter().any(|a| a == "--think");
    let request = args.last().cloned().unwrap_or_else(|| "Bold the header row.".into());

    // `--dump-prompt`: print the first prompt the app would send (for runtime benchmarks) and exit, no runtime needed
    if args.iter().any(|a| a == "--dump-prompt") {
        let mut host = lattice_mcp::Headless::new();
        if let Err(e) = lattice_assistant::ToolHost::call(&mut host, "new_workbook", &json!({"sample": sample})) {
            eprintln!("sample: {e}");
        }
        let h = Harness::default();
        let messages = [system_message(), user_message(&h.snapshot(&mut host), &request)];
        print!("{}", render(&messages, think, true));
        return;
    }

    let t0 = Instant::now();
    let cfg = RuntimeConfig {
        server: server.into(),
        model: model.into(),
        ctx_size: 16384,
        threads,
        gpu_layers,
        port: lattice_assistant::install::free_port().unwrap_or(18080),
    };
    let srv = match LlamaServer::spawn(&cfg, Duration::from_secs(300)) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("runtime: {e}");
            std::process::exit(1);
        }
    };
    println!("load_s {:.2}", t0.elapsed().as_secs_f64());

    let mut host = lattice_mcp::Headless::new();
    if let Err(e) = lattice_assistant::ToolHost::call(&mut host, "new_workbook", &json!({"sample": sample})) {
        eprintln!("sample: {e}");
    }
    let h = Harness::default();
    let generator: Arc<dyn Generator> = srv.clone();
    let t1 = Instant::now();
    let mut run = Run::start(generator, &h, &mut host, &request, think);
    let mut last = Instant::now();
    let mut first_text: Option<f64> = None;
    loop {
        for ev in run.poll(&h, &mut host) {
            match ev {
                Event::Text(_) => {
                    first_text.get_or_insert(t1.elapsed().as_secs_f64());
                }
                Event::ToolFinished { name, observation } => {
                    println!("tool {:>6.2}s {name}: {}", last.elapsed().as_secs_f64(), observation.lines().next().unwrap_or(""));
                    last = Instant::now();
                }
                Event::NeedsConfirmation { .. } => run.confirm(false),
                Event::Answer(a) => println!("answer {:>6.2}s {a}", last.elapsed().as_secs_f64()),
                Event::Warning(w) => println!("warning {w}"),
                Event::Failed(e) => println!("failed {e}"),
                Event::ToolStarted { .. } => {}
            }
        }
        if run.is_finished() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    println!(
        "total_s {:.2} first_token_s {:.2} prompt_tokens {} generated_tokens {}",
        t1.elapsed().as_secs_f64(),
        first_text.unwrap_or(-1.0),
        run.tokens_in,
        run.tokens_out
    );
}
