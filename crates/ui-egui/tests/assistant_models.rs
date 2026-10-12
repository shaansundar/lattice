//! The assistant's model commands validate their parameters and report state without opening dialogs. Only paths
//! that can't download or delete anything on the machine running the tests are exercised here; the downloader
//! itself is tested against a local server in `lattice-assistant`.
#![cfg(not(target_arch = "wasm32"))]

use lattice_engine::Session;
use lattice_ui_egui::{Services, SheetApp};
use serde_json::json;

fn app() -> SheetApp {
    let mut session = Session::new();
    session.new_workbook();
    SheetApp::new(session, Services::default())
}

#[test]
fn status_lists_both_models_and_a_recommendation() {
    let mut app = app();
    let s = app.run("assistant.models", json!({})).unwrap();
    let models = s["models"].as_array().unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(models.iter().filter(|m| m["recommended"] == json!(true)).count(), 1);
    for m in models {
        assert!(m["bytes"].as_u64().unwrap() > 1_000_000_000);
        assert_eq!(m["downloading"], json!(false));
    }
    assert!(["e4b", "e2b"].contains(&s["recommended"].as_str().unwrap()));
    assert!(!s["reason"].as_str().unwrap().is_empty());
    assert_eq!(s["runtime"], "notStarted");
}

#[test]
fn bad_parameters_are_errors_not_dialogs_or_crashes() {
    let mut app = app();
    for (id, p) in [
        ("assistant.install", json!({})),
        ("assistant.install", json!({"tier": "gpt"})),
        ("assistant.importModel", json!({"tier": "e2b"})),
        ("assistant.importModel", json!({"path": "/x.gguf"})),
        ("assistant.removeModel", json!({"tier": 7})),
        ("assistant.useModel", json!({})),
        ("assistant.cancelInstall", json!({"tier": "e9b"})),
        ("assistant.ask", json!({"prompt": "  "})),
    ] {
        assert!(app.run(id, p.clone()).is_err(), "{id} {p}");
    }
    assert_eq!(app.run("assistant.cancelInstall", json!({})).unwrap()["cancelled"], 0);
}

#[test]
fn manage_models_opens_the_pane() {
    let mut app = app();
    assert_eq!(app.run("assistant.manageModels", json!({"open": true})).unwrap()["open"], true);
    assert!(app.assistant.open);
    assert_eq!(app.run("assistant.manageModels", json!({})).unwrap()["open"], false);
}
