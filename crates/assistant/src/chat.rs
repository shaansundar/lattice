//! Gemma-4 chat rendering for the assistant's conversations (port of the parts of Google's canonical Gemma 4 chat
//! template, published 2026-07-09, that these conversations use: a system turn without tool declarations, user
//! turns, model turns with tool calls, the following role=tool messages, the thinking channel and the generation
//! prompt). Rendering in Lattice keeps the prompt independent of the runtime; `tests/golden/chat.json` holds renders
//! made by the reference tokenizer for real training rows.

use serde_json::Value;

use crate::pyfmt::py_str;

pub const BOS: &str = "<bos>";

fn role(m: &Value) -> &str {
    m.get("role").and_then(Value::as_str).unwrap_or("")
}

/// Jinja `dictsort` (case-insensitive by key; stable for equal keys).
fn dictsort(o: &serde_json::Map<String, Value>) -> Vec<(&String, &Value)> {
    let mut items: Vec<(&String, &Value)> = o.iter().collect();
    items.sort_by_key(|(k, _)| k.to_lowercase());
    items
}

/// The template's `format_argument`.
pub fn format_argument(v: &Value, escape_keys: bool) -> String {
    match v {
        Value::Null => "null".into(),
        Value::String(s) => format!("<|\"|>{s}<|\"|>"),
        Value::Bool(b) => if *b { "true" } else { "false" }.into(),
        Value::Object(o) => {
            let parts: Vec<String> = dictsort(o)
                .into_iter()
                .map(|(k, x)| format!("{}:{}", if escape_keys { format!("<|\"|>{k}<|\"|>") } else { k.clone() }, format_argument(x, escape_keys)))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        Value::Array(a) => format!("[{}]", a.iter().map(|x| format_argument(x, escape_keys)).collect::<Vec<_>>().join(",")),
        Value::Number(_) => py_str(v),
    }
}

/// The template's `strip_thinking`: drop `<|channel>…<channel|>` spans and trim.
fn strip_thinking(text: &str) -> String {
    let mut out = String::new();
    for part in text.split("<channel|>") {
        match part.split_once("<|channel>") {
            Some((before, _)) => out.push_str(before),
            None => out.push_str(part),
        }
    }
    out.trim().to_string()
}

fn content_text(m: &Value, model: bool) -> String {
    match m.get("content") {
        Some(Value::String(s)) => {
            if model {
                strip_thinking(s)
            } else {
                s.trim().to_string()
            }
        }
        Some(Value::Array(items)) => items
            .iter()
            .filter(|i| i.get("type").and_then(Value::as_str) == Some("text"))
            .map(|i| {
                let t = i.get("text").and_then(Value::as_str).unwrap_or("");
                if model { strip_thinking(t) } else { t.trim().to_string() }
            })
            .collect(),
        _ => String::new(),
    }
}

fn tool_response_block(name: &str, body: &Value) -> String {
    let inner = match body {
        Value::Object(o) => dictsort(o).into_iter().map(|(k, v)| format!("{k}:{}", format_argument(v, false))).collect::<Vec<_>>().join(","),
        other => format!("value:{}", format_argument(other, false)),
    };
    format!("<|tool_response>response:{name}{{{inner}}}<tool_response|>")
}

#[derive(PartialEq, Clone, Copy)]
enum Prev {
    None,
    Think,
    ToolCall,
    ToolResponse,
}

/// Render `messages` (system first, then user / assistant / tool) exactly as the reference chat template does.
pub fn render(messages: &[Value], enable_thinking: bool, add_generation_prompt: bool) -> String {
    let mut out = String::from(BOS);
    let mut prev = Prev::None;
    let mut loop_messages = messages;
    let first_is_system = messages.first().is_some_and(|m| matches!(role(m), "system" | "developer"));
    if enable_thinking || first_is_system {
        out.push_str("<|turn>system\n");
        if enable_thinking {
            out.push_str("<|think|>\n");
            prev = Prev::Think;
        }
        if first_is_system {
            if let Some(m) = messages.first() {
                match m.get("content") {
                    Some(Value::String(s)) => out.push_str(s.trim()),
                    Some(Value::Array(items)) => {
                        for i in items {
                            out.push_str(i.get("text").and_then(Value::as_str).unwrap_or("").trim());
                            out.push(' ');
                        }
                    }
                    _ => {}
                }
            }
            loop_messages = messages.get(1..).unwrap_or(&[]);
        }
        out.push_str("<turn|>\n");
    }
    let last_user = loop_messages.iter().rposition(|m| role(m) == "user").map_or(-1, |i| i as i64);
    let mut prev_non_tool_role: Option<&str> = None;
    for (idx, m) in loop_messages.iter().enumerate() {
        if role(m) == "tool" {
            continue;
        }
        prev = Prev::None;
        let r = if role(m) == "assistant" { "model" } else { role(m) };
        let continue_same = r == "model" && prev_non_tool_role == Some("assistant");
        if !continue_same {
            out.push_str("<|turn>");
            out.push_str(r);
            out.push('\n');
        }
        let thinking = [m.get("reasoning"), m.get("reasoning_content")].into_iter().flatten().find_map(|t| t.as_str().filter(|s| !s.is_empty()));
        if let Some(t) = thinking
            && (idx as i64) > last_user
        {
            out.push_str("<|channel>thought\n");
            out.push_str(t);
            out.push_str("\n<channel|>");
        }
        let calls = m.get("tool_calls").and_then(Value::as_array).filter(|c| !c.is_empty());
        if let Some(calls) = calls {
            for tc in calls {
                let f = tc.get("function").unwrap_or(&Value::Null);
                out.push_str("<|tool_call>call:");
                out.push_str(f.get("name").and_then(Value::as_str).unwrap_or(""));
                out.push('{');
                if let Some(args) = f.get("arguments").and_then(Value::as_object) {
                    let parts: Vec<String> = dictsort(args).into_iter().map(|(k, v)| format!("{k}:{}", format_argument(v, false))).collect();
                    out.push_str(&parts.join(","));
                }
                out.push_str("}<tool_call|>");
            }
            prev = Prev::ToolCall;
        }
        let mut tr_out = false;
        if let Some(calls) = calls {
            for follow in loop_messages.iter().skip(idx + 1).take_while(|f| role(f) == "tool") {
                let mut name = follow.get("name").and_then(Value::as_str).filter(|s| !s.is_empty()).unwrap_or("unknown").to_string();
                for tc in calls {
                    if tc.get("id").is_some() && tc.get("id") == follow.get("tool_call_id") {
                        name = tc.get("function").and_then(|f| f.get("name")).and_then(Value::as_str).unwrap_or("").to_string();
                    }
                }
                let body = match follow.get("content") {
                    Some(Value::Array(parts)) => Value::String(
                        parts
                            .iter()
                            .filter(|p| p.get("type").and_then(Value::as_str) == Some("text"))
                            .map(|p| p.get("text").and_then(Value::as_str).unwrap_or(""))
                            .collect(),
                    ),
                    Some(v) => v.clone(),
                    None => Value::Null,
                };
                out.push_str(&tool_response_block(&name, &body));
                tr_out = true;
                prev = Prev::ToolResponse;
            }
        }
        let captured = content_text(m, r == "model");
        out.push_str(&captured);
        let has_content = !captured.trim().is_empty();
        let next_nt = loop_messages.iter().skip(idx + 1).find(|x| role(x) != "tool").map(role);
        let continues_into_next = r == "model" && next_nt == Some("assistant") && (calls.is_none() || tr_out);
        if prev == Prev::ToolCall && !tr_out {
            out.push_str("<|tool_response>");
        } else if continues_into_next {
        } else if !(tr_out && !has_content && next_nt.is_none()) {
            out.push_str("<turn|>\n");
        }
        prev_non_tool_role = Some(role(m));
    }
    if add_generation_prompt {
        if prev != Prev::ToolResponse && prev != Prev::ToolCall {
            out.push_str("<|turn>model\n");
        } else if prev == Prev::ToolResponse && enable_thinking {
            out.push_str("<|channel>thought\n");
        }
    }
    out
}
