//! One model tool call → guardrails → the engine → one observation (port of `HarnessBackend.run`).

use serde_json::{Value, json};

use crate::docs::DocsIndex;
use crate::observe::{fmt_scalar, render_error, render_read, render_result};
use crate::prompt::{TOOL_NAMES, context_block, preview_range};

/// Runs MCP tools against a workbook session. `Ok` is the tool's JSON result, `Err` its error message
/// (e.g. `execute_command: unknown command \`x\``).
pub trait ToolHost {
    fn call(&mut self, name: &str, args: &Value) -> Result<Value, String>;
}

/// The lattice-mcp tool layer over an in-process session (what the app and tests use).
impl ToolHost for lattice_mcp::Headless {
    fn call(&mut self, name: &str, args: &Value) -> Result<Value, String> {
        lattice_mcp::check_arguments(name, args)?;
        let r = lattice_mcp::call_tool(self, name, args);
        let text: String = r.content.iter().filter_map(|c| c.get("text").and_then(Value::as_str)).collect();
        if r.is_error {
            return Err(text);
        }
        Ok(serde_json::from_str(&text).unwrap_or(Value::String(text)))
    }
}

pub const MAX_CELLS_PER_WRITE: usize = 5000;

/// File, app and window commands, sheet deletion and the workbook-file tools need the user's confirmation.
pub fn is_risky(name: &str, args: &Value) -> bool {
    if matches!(name, "open_workbook" | "save_workbook" | "new_workbook") {
        return true;
    }
    if name != "execute_command" {
        return false;
    }
    let cmd = args.get("command").map(crate::pyfmt::py_str).unwrap_or_else(|| "None".into());
    cmd.starts_with("file.") || cmd.starts_with("app.") || cmd.starts_with("window.") || cmd == "home.deleteSheet" || cmd == "sheet.delete"
}

pub struct Harness {
    pub docs: DocsIndex,
}

impl Default for Harness {
    fn default() -> Self {
        Harness { docs: DocsIndex::bundled() }
    }
}

impl Harness {
    /// Observation for one tool call. `confirm` is asked before risky calls (the app shows a one-click prompt).
    pub fn run(&self, host: &mut dyn ToolHost, name: &str, args: &Value, confirm: &mut dyn FnMut(&str, &Value) -> bool) -> String {
        let empty = json!({});
        let args = if args.is_object() { args } else { &empty };
        if name == "lattice_docs" {
            return self.docs.render(&args.get("search").map(crate::pyfmt::py_str).unwrap_or_default());
        }
        if !TOOL_NAMES.contains(&name) {
            return render_error(&format!("unknown tool `{name}`"));
        }
        if is_risky(name, args) && !confirm(name, args) {
            let what = if name == "execute_command" {
                args.get("command").map(crate::pyfmt::py_str).unwrap_or_else(|| "None".into())
            } else {
                name.to_string()
            };
            return format!("DECLINED: the user did not confirm {what}. Tell them what you would do instead of retrying.");
        }
        if name == "write_range" {
            let n: usize =
                args.get("values").and_then(Value::as_array).map(|rows| rows.iter().map(|r| r.as_array().map_or(1, Vec::len)).sum()).unwrap_or(0);
            if n > MAX_CELLS_PER_WRITE {
                return format!("ERROR: write_range is limited to {MAX_CELLS_PER_WRITE} cells per call; use fill commands for large ranges");
            }
        }
        if name == "read_range" {
            let mut a = serde_json::Map::new();
            for k in ["range", "sheet"] {
                if let Some(v) = args.get(k) {
                    a.insert(k.into(), v.clone());
                }
            }
            let mut f = a.clone();
            f.insert("formulas".into(), json!(true));
            let mut d = a;
            d.insert("formatted".into(), json!(true));
            return match host.call("read_range", &Value::Object(f)) {
                Err(e) => render_error(&e),
                Ok(rf) => render_read(&rf, host.call("read_range", &Value::Object(d)).ok().as_ref()),
            };
        }
        match host.call(name, args) {
            Ok(v) => render_result(name, &v),
            Err(e) => render_error(&e),
        }
    }

    /// The `<workbook>` snapshot that starts every request.
    pub fn snapshot(&self, host: &mut dyn ToolHost) -> String {
        let Ok(wb) = host.call("inspect_workbook", &json!({})) else { return "<workbook>\n(unavailable)\n</workbook>".into() };
        let rows = |v: &Value| -> Vec<Vec<Value>> {
            v.get("values")
                .and_then(Value::as_array)
                .map(|r| r.iter().map(|x| x.as_array().cloned().unwrap_or_default()).collect())
                .unwrap_or_default()
        };
        if let Some((sheet, rng)) = preview_range(&wb)
            && let Ok(rf) = host.call("read_range", &json!({"range": rng, "formulas": true}))
        {
            let rv = host.call("read_range", &json!({"range": rng, "formatted": true})).ok();
            let top_left = rng.split(':').next().unwrap_or("A1").to_string();
            let (f, v) = (rows(&rf), rv.as_ref().map(rows).unwrap_or_default());
            return context_block(&wb, Some((&sheet, &top_left, &f, &v)));
        }
        context_block(&wb, None)
    }
}

/// `fmt_scalar` re-exported for hosts that render their own status lines.
pub fn short(v: &Value) -> String {
    fmt_scalar(v, 80)
}
