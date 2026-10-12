//! Tool results → the compact text the model sees (port of the training harness, `lattice_harness/backend.py`).
//!
//! The model was trained on exactly these renderings, so any change here must be made in the Python reference too
//! (golden fixtures in `tests/golden/observe.json` keep the two byte-identical).

use serde_json::Value;

use crate::pyfmt::{float_repr, json_dumps, py_str, round6, truncate};

pub const MAX_ROWS: usize = 40;
pub const MAX_COLS: usize = 15;
pub const MAX_LIST: usize = 12;

/// Python `fmt_scalar`: one cell value as grid text.
pub fn fmt_scalar(v: &Value, max_chars: usize) -> String {
    let s = match v {
        Value::Null => return String::new(),
        Value::Object(o) if o.contains_key("error") => return o.get("error").map(py_str).unwrap_or_default(),
        Value::Bool(b) => if *b { "TRUE" } else { "FALSE" }.to_string(),
        Value::Number(n) if n.as_i64().is_none() && n.as_u64().is_none() => {
            let f = n.as_f64().unwrap_or(0.0);
            if f.fract() == 0.0 && f.abs() < 1e15 { format!("{}", f as i64) } else { float_repr(round6(f)) }
        }
        other => py_str(other),
    };
    truncate(&s.replace('\n', " ").replace('|', "/"), max_chars)
}

pub fn col_letter(mut n: u32) -> String {
    let mut s = Vec::new();
    while n > 0 {
        let r = (n - 1) % 26;
        n = (n - 1) / 26;
        s.push(char::from(b'A' + r as u8));
    }
    s.iter().rev().collect()
}

/// "B4" → (2, 4); None for anything else (absolute `$` markers allowed).
pub fn split_ref(r: &str) -> Option<(u32, u32)> {
    let r = r.trim();
    let letters: String = r.chars().filter(|c| *c != '$').take_while(char::is_ascii_uppercase).collect();
    let rest: String = r.chars().filter(|c| *c != '$').skip(letters.len()).collect();
    if letters.is_empty() || letters.len() > 3 || rest.is_empty() || !rest.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let col = letters.bytes().fold(0u32, |a, b| a.saturating_mul(26).saturating_add(u32::from(b - b'A' + 1)));
    Some((col, rest.parse().ok()?))
}

fn rows_of(v: Option<&Value>) -> Vec<Vec<Value>> {
    v.and_then(Value::as_array).map(|rows| rows.iter().map(|r| r.as_array().cloned().unwrap_or_default()).collect()).unwrap_or_default()
}

/// Grid of a range: formula cells show `=FORMULA→displayed value`; clipped to 40 rows × 15 columns.
pub fn render_grid(sheet: &str, top_left: &str, formulas: &[Vec<Value>], values: &[Vec<Value>]) -> String {
    let (c0, r0) = split_ref(top_left).unwrap_or((1, 1));
    let widest = formulas.iter().map(Vec::len).max().unwrap_or(0);
    let ncols = widest.min(MAX_COLS);
    let rows = &formulas[..formulas.len().min(MAX_ROWS)];
    let clipped = formulas.len() > MAX_ROWS || widest > MAX_COLS;
    let last_col = col_letter(c0 + ncols.max(1) as u32 - 1);
    let last_row = r0 as usize + rows.len().max(1) - 1;
    let mut lines = vec![format!("'{sheet}'!{top_left}:{last_col}{last_row}{}", if clipped { " (clipped to 40 rows x 15 columns)" } else { "" })];
    let heads: Vec<String> = (0..ncols).map(|j| col_letter(c0 + j as u32)).collect();
    lines.push(format!("| row | {} |", heads.join(" | ")));
    let empty = Vec::new();
    for (i, frow) in rows.iter().enumerate() {
        let vrow = values.get(i).unwrap_or(&empty);
        let cells: Vec<String> = (0..ncols)
            .map(|j| {
                let f = frow.get(j).unwrap_or(&Value::Null);
                let v = vrow.get(j).unwrap_or(&Value::Null);
                match f.as_str() {
                    Some(fs) if fs.starts_with('=') => format!("{}→{}", fmt_scalar(f, 60), fmt_scalar(v, 24)),
                    _ => fmt_scalar(if v.is_null() { f } else { v }, 40),
                }
            })
            .collect();
        lines.push(format!("| {} | {} |", r0 as usize + i, cells.join(" | ")));
    }
    lines.join("\n")
}

/// read_range observation from the formulas read and the formatted read (as the harness issues them).
pub fn render_read(formulas: &Value, formatted: Option<&Value>) -> String {
    let rng = formulas.get("range").map(py_str).unwrap_or_default();
    let first = rng.split(':').next().unwrap_or("");
    let top_left = first.rsplit('!').next().filter(|s| !s.is_empty()).unwrap_or("A1");
    let sheet = formulas.get("sheet").and_then(Value::as_str).unwrap_or("");
    render_grid(sheet, top_left, &rows_of(formulas.get("values")), &rows_of(formatted.and_then(|f| f.get("values"))))
}

fn color(c: &Value) -> Option<String> {
    if let Some(rgb) = c.get("Rgb").and_then(Value::as_i64) {
        return Some(format!("#{rgb:06X}"));
    }
    c.get("Theme").map(|t| format!("theme{}", py_str(t)))
}

fn truthy(v: Option<&Value>) -> bool {
    match v {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(Value::Number(n)) => n.as_f64().is_some_and(|f| f != 0.0),
        Some(Value::String(s)) => !s.is_empty(),
        Some(Value::Array(a)) => !a.is_empty(),
        Some(Value::Object(o)) => !o.is_empty(),
    }
}

/// Non-default style bits of one cell (font, fill, number format, alignment, borders).
pub fn style_summary(st: &Value) -> String {
    let mut out = Vec::new();
    let font = st.get("font").cloned().unwrap_or(Value::Null);
    let mut bits: Vec<String> = ["bold", "italic", "strike"].iter().filter(|k| truthy(font.get(**k))).map(|k| k.to_string()).collect();
    if !matches!(font.get("underline").and_then(Value::as_str), None | Some("None")) {
        bits.push("underline".into());
    }
    let name = font.get("name").and_then(Value::as_str);
    let size = font.get("size").and_then(Value::as_f64);
    if let Some(n) = name
        && !n.is_empty()
        && (n != "Calibri" || size != Some(11.0))
    {
        bits.push(format!("{n} {}", fmt_scalar(font.get("size").unwrap_or(&Value::Null), 40)));
    }
    if let Some(c) = font.get("color").and_then(color) {
        bits.push(format!("color {c}"));
    }
    if !bits.is_empty() {
        out.push(format!("font {}", bits.join(" ")));
    }
    let fill = st.get("fill").cloned().unwrap_or(Value::Null);
    if !matches!(fill.get("pattern").and_then(Value::as_str), None | Some("None"))
        && let Some(c) = fill.get("fg").and_then(color)
    {
        out.push(format!("fill {c}"));
    }
    let nf = st.get("num_fmt").filter(|v| truthy(Some(v))).or_else(|| st.get("numFmt"));
    if let Some(nf) = nf.filter(|v| truthy(Some(v)) && v.as_str() != Some("General")) {
        out.push(format!("format {}", json_dumps(nf, false)));
    }
    let al = st.get("align").cloned().unwrap_or(Value::Null);
    if let Some(h) = al.get("h").filter(|h| !h.is_null() && h.as_str() != Some("General")) {
        out.push(format!("align {}", py_str(h).to_lowercase()));
    }
    if truthy(al.get("wrap")) {
        out.push("wrap".into());
    }
    let mut edges: Vec<String> = st
        .get("borders")
        .and_then(Value::as_object)
        .map(|b| {
            b.iter()
                .filter(|(_, line)| line.is_object() && !matches!(line.get("style").and_then(Value::as_str), None | Some("None")))
                .map(|(e, _)| e.clone())
                .collect()
        })
        .unwrap_or_default();
    if !edges.is_empty() {
        edges.sort();
        out.push(format!("borders {}", edges.join(",")));
    }
    out.join("; ")
}

/// get_cell observation.
pub fn render_cell(r: &Value) -> String {
    let mut parts = Vec::new();
    if truthy(r.get("formula")) {
        parts.push(format!("formula {}", py_str(&r["formula"])));
    }
    let value = r.get("value").unwrap_or(&Value::Null);
    let shown = fmt_scalar(value, 80);
    parts.push(format!("value {}", if shown.is_empty() { "(empty)".to_string() } else { shown }));
    if let Some(d) = r.get("display")
        && !d.is_null()
        && d.as_str() != Some("")
        && py_str(d) != fmt_scalar(value, 40)
    {
        parts.push(format!("shows {}", json_dumps(d, false)));
    }
    for k in ["spill", "spilledFrom", "merge", "comment", "hyperlink"] {
        if let Some(v) = r.get(k).filter(|v| truthy(Some(v))) {
            let text = if v.is_object() { Value::String(json_dumps(v, false)) } else { v.clone() };
            parts.push(format!("{k} {}", fmt_scalar(&text, 80)));
        }
    }
    let st = style_summary(r.get("style").unwrap_or(&Value::Null));
    if !st.is_empty() {
        parts.push(format!("style {st}"));
    }
    format!("{}: {}", r.get("cell").map(py_str).unwrap_or_else(|| "None".into()), parts.join(" | "))
}

/// inspect_workbook observation; `undo` includes the last few undo labels (the snapshot block leaves them out).
pub fn render_workbook(w: &Value, undo: bool) -> String {
    let empty = Vec::new();
    let sheets = w.get("sheets").and_then(Value::as_array).unwrap_or(&empty);
    let active_i = w.get("activeSheet").and_then(Value::as_u64).unwrap_or(0) as usize;
    let active = sheets.get(active_i).and_then(|s| s.get("name")).map(py_str).unwrap_or_else(|| "?".into());
    let title = [w.get("title"), w.get("path")].into_iter().flatten().find(|v| truthy(Some(v))).cloned().unwrap_or(Value::String("untitled".into()));
    let get = |k: &str| w.get(k).map(py_str).unwrap_or_else(|| "None".into());
    let calc = w.get("calcMode").filter(|c| !c.is_null() && c.as_str() != Some("Automatic"));
    let mut lines = vec![format!(
        "Workbook {}{} · active sheet '{active}' · selection {} (active cell {}){}",
        json_dumps(&title, false),
        if truthy(w.get("dirty")) { " (unsaved changes)" } else { "" },
        get("selection"),
        get("activeCell"),
        calc.map(|c| format!(" · calc {}", py_str(c))).unwrap_or_default()
    )];
    for s in sheets {
        let sget = |k: &str| s.get(k).map(py_str).unwrap_or_else(|| "None".into());
        let used = s.get("usedRange").filter(|v| truthy(Some(v))).map(py_str).unwrap_or_else(|| "(empty)".into());
        let mut bits = vec![format!("- '{}' used {used}", sget("name"))];
        if let Some(v) = s.get("visibility").filter(|v| !v.is_null() && v.as_str() != Some("Visible")) {
            bits.push(py_str(v).to_lowercase());
        }
        for t in s.get("tables").and_then(Value::as_array).unwrap_or(&empty) {
            let cols: Vec<String> = t.get("columns").and_then(Value::as_array).unwrap_or(&empty).iter().map(py_str).collect();
            let tget = |k: &str| t.get(k).map(py_str).unwrap_or_else(|| "None".into());
            bits.push(format!("table {} {} [{}]", tget("name"), tget("range"), cols.join(", ")));
        }
        let charts: Vec<String> = s
            .get("charts")
            .and_then(Value::as_array)
            .unwrap_or(&empty)
            .iter()
            .map(|c| {
                let title = c.get("title").filter(|v| truthy(Some(v))).cloned().unwrap_or(Value::String(String::new()));
                format!(
                    "#{} {} {} at {}",
                    c.get("id").map(py_str).unwrap_or_else(|| "None".into()),
                    c.get("kind").map(py_str).unwrap_or_default().to_lowercase(),
                    json_dumps(&title, false),
                    c.get("at").map(py_str).unwrap_or_else(|| "None".into())
                )
            })
            .collect();
        if !charts.is_empty() {
            bits.push(format!("charts: {}", charts.join(", ")));
        }
        if truthy(s.get("autofilter")) {
            bits.push(format!("autofilter {}", sget("autofilter")));
        }
        if let Some(fr) = s.get("freeze").and_then(Value::as_array)
            && fr.len() == 2
            && fr.iter().any(|x| truthy(Some(x)))
        {
            bits.push(format!("frozen {} rows, {} columns", py_str(&fr[0]), py_str(&fr[1])));
        }
        for (k, label) in [
            ("conditionalFormats", "conditional formats"),
            ("validations", "validations"),
            ("comments", "comments"),
            ("images", "images"),
            ("shapes", "shapes"),
        ] {
            if truthy(s.get(k)) {
                bits.push(format!("{label} {}", sget(k)));
            }
        }
        if let Some(m) = s.get("merges").and_then(Value::as_array).filter(|m| !m.is_empty()) {
            bits.push(format!("merged {}", m.iter().take(6).map(py_str).collect::<Vec<_>>().join(", ")));
        }
        if truthy(s.get("protected")) {
            bits.push("protected".into());
        }
        lines.push(bits.join(" · "));
    }
    if let Some(names) = w.get("names").and_then(Value::as_array).filter(|n| !n.is_empty()) {
        let shown: Vec<String> = names
            .iter()
            .take(12)
            .map(|n| match n {
                Value::Object(o) => format!(
                    "{}={}",
                    o.get("name").map(py_str).unwrap_or_else(|| "None".into()),
                    o.get("refersTo").or_else(|| o.get("formula")).map(py_str).unwrap_or_default()
                ),
                other => py_str(other),
            })
            .collect();
        lines.push(format!("Names: {}", shown.join(", ")));
    }
    if undo && let Some(u) = w.get("undo").and_then(Value::as_array).filter(|u| !u.is_empty()) {
        let tail: Vec<String> = u.iter().skip(u.len().saturating_sub(4)).map(py_str).collect();
        lines.push(format!("Undo history (latest last): {}", tail.join(", ")));
    }
    lines.join("\n")
}

fn render_list(rows: &[Value], line: impl Fn(&Value) -> String, none: &str) -> String {
    if rows.is_empty() {
        return none.to_string();
    }
    let mut out: Vec<String> = rows.iter().take(MAX_LIST).map(line).collect();
    if rows.len() > MAX_LIST {
        out.push(format!("(+{} more; narrow the search)", rows.len() - MAX_LIST));
    }
    out.join("\n")
}

/// list_commands observation.
pub fn render_commands(rows: &[Value]) -> String {
    render_list(
        rows,
        |c| {
            let menu: Vec<String> = c.get("menu").and_then(Value::as_array).map(|m| m.iter().map(py_str).collect()).unwrap_or_default();
            let path = if menu.is_empty() { String::new() } else { format!(" [{}]", menu.join(" > ")) };
            format!(
                "{} — {}{path} params {}",
                c.get("id").map(py_str).unwrap_or_else(|| "None".into()),
                c.get("label").map(py_str).unwrap_or_else(|| "None".into()),
                fmt_scalar(c.get("params").unwrap_or(&Value::Null), 220)
            )
        },
        "No matching commands.",
    )
}

/// list_functions observation.
pub fn render_functions(rows: &[Value]) -> String {
    render_list(
        rows,
        |f| {
            format!(
                "{} — {}",
                f.get("signature").map(py_str).unwrap_or_else(|| "None".into()),
                fmt_scalar(f.get("description").unwrap_or(&Value::Null), 160)
            )
        },
        "No matching functions.",
    )
}

/// Any other successful tool result.
pub fn render_ok(v: &Value) -> String {
    if v.is_null() { "OK".into() } else { format!("OK {}", truncate(&json_dumps(v, true), 400)) }
}

/// A failed tool call: the engine's message without the leading `tool_name: ` (as the model saw it in training).
pub fn render_error(msg: &str) -> String {
    let strip = regex::Regex::new(r"^(tools/call: )?[a-z_]+: ").map(|re| re.replace(msg, "").into_owned()).unwrap_or_else(|_| msg.to_string());
    format!("ERROR: {}", fmt_scalar(&Value::String(strip), 300))
}

/// Observation for a tool that returned `result` (already parsed JSON).
pub fn render_result(tool: &str, result: &Value) -> String {
    let as_list = |k: &str| -> Vec<Value> {
        match result {
            Value::Array(a) => a.clone(),
            other => other.get(k).and_then(Value::as_array).cloned().unwrap_or_default(),
        }
    };
    match tool {
        "get_cell" => render_cell(result),
        "inspect_workbook" => render_workbook(result, true),
        "list_commands" => render_commands(&as_list("commands")),
        "list_functions" => render_functions(&as_list("functions")),
        "evaluate_formula" => format!(
            "{} = {}",
            result.get("formula").map(py_str).unwrap_or_else(|| "None".into()),
            fmt_scalar(result.get("result").unwrap_or(&Value::Null), 200)
        ),
        _ => render_ok(result),
    }
}
