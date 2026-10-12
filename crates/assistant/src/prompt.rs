//! The assistant's prompt: a fixed system message (cached by the runtime once per session) and per request a user
//! message that starts with a compact snapshot of the workbook. Byte-identical to the training harness
//! (`lattice_harness/prompt.py`); `tests/golden/prompt.json` guards it.

use serde_json::{Value, json};

use crate::observe::{col_letter, render_grid, render_workbook, split_ref};

pub const PREVIEW_ROWS: u32 = 12;
pub const PREVIEW_COLS: u32 = 10;

/// The system prompt the model was trained with, including the compact list of its 15 tools.
pub const SYSTEM: &str = "You are the assistant built into Lattice, a spreadsheet app. You work on the user's open workbook with tools; your changes apply directly and the user can undo each request in one step.
- The message starts with a snapshot of the workbook. Read more with read_range or inspect_workbook when you need it.
- Prefer formulas over typed results so values stay live, and change only the cells the request is about.
- Use the specific tools when they fit; every other Lattice action is execute_command with a command id (look ids and params up with list_commands). Use only worksheet functions Lattice has (list_functions).
- Questions about Lattice itself (features, shortcuts, lattice-cli, MCP, scripting): search lattice_docs.
- Questions about the data: read or compute what you need (evaluate_formula) and answer with the numbers.
- Finish with one or two plain sentences: what you changed, or the answer. No tool call after that.
Tools:
- inspect_workbook() — Summary of the workbook: sheets, used ranges, tables, charts, names, selection.
- read_range(range?, sheet?) — Read cells as a grid; formula cells show =FORMULA→value. Default: the used range.
- get_cell(cell) — One cell: value, formula, display text, style.
- set_cell(cell, input) — Enter one cell as if typed: number, text or =formula.
- write_range(range, values) — Write rows of values from the top-left cell of range; strings starting with = are formulas.
- evaluate_formula(formula, cell?) — Compute a formula without changing any cell.
- format_range(range, bold?, italic?, underline?, fontName?, fontSize?, fontColor?, fillColor?, numberFormat?, horizontalAlign?, wrapText?, borders?) — Apply several formats to a range in one call.
- sort_range(keys, range?, header?) — Sort rows by one or more columns, most significant first.
- filter(column, range?, values?, custom?, top?, blanks?, clear?) — Filter rows by a column (turns on AutoFilter when needed).
- create_table(range, header?, name?, style?) — Format a range as a table (structured references, banded rows, filter buttons).
- insert_chart(range, type, subtype?, title?, at?) — Insert a chart from a data range including headers.
- execute_command(command, params?) — Run any Lattice command by id (every ribbon/menu action, undo, files). Look ids and params up with list_commands. file.*, app.*, window.* and sheet deletion ask the user first.
- list_commands(search?) — Search Lattice commands: id, label, ribbon path, params.
- list_functions(search?, category?) — Search worksheet functions Lattice supports: signature and description.
- lattice_docs(search) — Search Lattice's documentation: features, shortcuts, lattice-cli, MCP, scripting, file formats.";

/// The tools the model may call (everything else is refused before it reaches the workbook).
pub const TOOL_NAMES: [&str; 15] = [
    "inspect_workbook",
    "read_range",
    "get_cell",
    "set_cell",
    "write_range",
    "evaluate_formula",
    "format_range",
    "sort_range",
    "filter",
    "create_table",
    "insert_chart",
    "execute_command",
    "list_commands",
    "list_functions",
    "lattice_docs",
];

pub fn system_message() -> Value {
    json!({"role": "system", "content": SYSTEM})
}

/// Top-left PREVIEW_ROWS × PREVIEW_COLS of the active sheet's used range: (sheet name, "B2:I13").
pub fn preview_range(workbook: &Value) -> Option<(String, String)> {
    let i = workbook.get("activeSheet").and_then(Value::as_u64).unwrap_or(0) as usize;
    let sheet = workbook.get("sheets")?.as_array()?.get(i)?;
    let used = sheet.get("usedRange")?.as_str().filter(|u| !u.is_empty())?;
    let (a, b) = used.split_once(':').unwrap_or((used, used));
    let (c0, r0) = split_ref(a).unwrap_or((1, 1));
    let (c1, r1) = split_ref(if b.is_empty() { a } else { b }).unwrap_or((c0, r0));
    let c1 = c1.min(c0 + PREVIEW_COLS - 1);
    let r1 = r1.min(r0 + PREVIEW_ROWS - 1);
    Some((sheet.get("name")?.as_str()?.to_string(), format!("{}{r0}:{}{r1}", col_letter(c0), col_letter(c1))))
}

/// `<workbook>…</workbook>` block from an inspect result and an optional preview (sheet, top-left, formulas, values).
pub fn context_block(workbook: &Value, preview: Option<(&str, &str, &[Vec<Value>], &[Vec<Value>])>) -> String {
    let mut parts = vec!["<workbook>".to_string(), render_workbook(workbook, false)];
    if let Some((sheet, top_left, formulas, values)) = preview {
        parts.push(render_grid(sheet, top_left, formulas, values));
    }
    parts.push("</workbook>".into());
    parts.join("\n")
}

pub fn user_message(context: &str, request: &str) -> Value {
    json!({"role": "user", "content": format!("{context}\n\n{}", request.trim())})
}
