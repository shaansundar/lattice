//! Byte-for-byte parity with the Python harness the model was trained on (fixtures from `scripts/make_golden.py`
//! in the training repo, recorded against a real headless Lattice session).

use lattice_assistant::docs::DocsIndex;
use lattice_assistant::observe::{render_error, render_read, render_result};
use lattice_assistant::prompt::{SYSTEM, context_block};
use lattice_assistant::{is_risky, parse_tool_calls, split_thinking};
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let path = format!("{}/tests/golden/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap()
}

fn rows(v: &Value) -> Vec<Vec<Value>> {
    v.as_array().unwrap().iter().map(|r| r.as_array().unwrap().clone()).collect()
}

#[test]
fn observations_match_training() {
    let f = fixture("observe.json");
    for case in f["observe"].as_array().unwrap() {
        let tool = case["tool"].as_str().unwrap();
        let want = case["text"].as_str().unwrap();
        let got = if tool == "read_range" {
            if case["ok"].as_bool().unwrap() {
                render_read(&case["raw"], if case["raw_formatted"].is_null() { None } else { Some(&case["raw_formatted"]) })
            } else {
                render_error(case["raw"].as_str().unwrap())
            }
        } else if case["ok"].as_bool().unwrap() {
            render_result(tool, &case["raw"])
        } else {
            // The Python client prefixed JSON-RPC errors with the method; the stripping rule removes it either way.
            render_error(case["raw"].as_str().unwrap())
        };
        assert_eq!(got, want, "tool {tool} {}", case["args"]);
    }
    for case in f["risky"].as_array().unwrap() {
        assert_eq!(is_risky(case["tool"].as_str().unwrap(), &case["args"]), case["risky"].as_bool().unwrap(), "{case}");
    }
}

#[test]
fn system_prompt_and_snapshot_match_training() {
    let f = fixture("prompt.json");
    assert_eq!(SYSTEM, f["system"].as_str().unwrap());
    let c = &f["context"];
    let top_left = c["preview_range"].as_str().unwrap().split(':').next().unwrap().to_string();
    let sheet = c["workbook"]["sheets"][c["workbook"]["activeSheet"].as_u64().unwrap() as usize]["name"].as_str().unwrap().to_string();
    let (fo, va) = (rows(&c["formulas"]), rows(&c["formatted"]));
    assert_eq!(context_block(&c["workbook"], Some((&sheet, &top_left, &fo, &va))), c["text"].as_str().unwrap());
    assert_eq!(lattice_assistant::prompt::preview_range(&c["workbook"]).map(|p| p.1).as_deref(), c["preview_range"].as_str());
}

#[test]
fn tool_call_parsing_matches_training() {
    for case in fixture("parse.json").as_array().unwrap() {
        let text = case["text"].as_str().unwrap();
        let got: Vec<Value> = parse_tool_calls(text).into_iter().map(|c| serde_json::json!({"name": c.name, "arguments": c.arguments})).collect();
        assert_eq!(Value::Array(got), case["calls"], "{text}");
        let (t, fin, closed) = split_thinking(text);
        assert_eq!(serde_json::json!([t, fin, closed]), case["thinking"], "{text}");
    }
}

#[test]
fn docs_search_matches_training() {
    // Same chunking and BM25 as the training harness, on the doc files the fixture was built from.
    let f = fixture("docs.json");
    let chunks: Vec<(String, String)> = f["files"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|nt| lattice_assistant::docs::chunk_markdown(nt[0].as_str().unwrap(), nt[1].as_str().unwrap()))
        .collect();
    let d = DocsIndex::new(chunks);
    assert_eq!(d.len() as u64, f["chunks"].as_u64().unwrap());
    for q in f["queries"].as_array().unwrap() {
        assert_eq!(d.render(q["q"].as_str().unwrap()), q["text"].as_str().unwrap(), "query {}", q["q"]);
    }
    // The docs bundled into this build index and answer too.
    assert!(!DocsIndex::bundled().is_empty());
    assert!(DocsIndex::bundled().render("control port").contains("7979"));
}

#[test]
fn harness_runs_real_tools_end_to_end() {
    use lattice_assistant::{Harness, ToolHost};
    let mut host = lattice_mcp::Headless::new();
    host.call("new_workbook", &serde_json::json!({"sample": "sales"})).unwrap();
    let h = Harness::default();
    let mut yes = |_: &str, _: &Value| true;
    let mut no = |_: &str, _: &Value| false;
    assert!(h.snapshot(&mut host).starts_with("<workbook>\nWorkbook \"Quarterly Sales Dashboard\""));
    assert!(h.run(&mut host, "read_range", &serde_json::json!({"range": "B4:C5"}), &mut yes).contains("| 5 | North | $48,200 |"));
    assert_eq!(h.run(&mut host, "set_cell", &serde_json::json!({"cell": "K1", "input": "=1+2"}), &mut yes), r#"OK {"cell":"K1","value":"3"}"#);
    assert_eq!(h.run(&mut host, "set_cell", &serde_json::json!({"input": "5"}), &mut yes), "ERROR: missing required argument `cell`");
    assert!(
        h.run(&mut host, "execute_command", &serde_json::json!({"command": "file.new"}), &mut no)
            .starts_with("DECLINED: the user did not confirm file.new")
    );
    assert!(h.run(&mut host, "screenshot", &serde_json::json!({"path": "x"}), &mut yes).starts_with("ERROR: unknown tool"));
    assert!(h.run(&mut host, "lattice_docs", &serde_json::json!({"search": "control port"}), &mut yes).contains("--control 7979"));
}
