//! Gemma-4 output parsing: native tool calls (`<|tool_call>call:NAME{k:v}<tool_call|>`) and the thinking channel
//! (`<|channel>thought … <channel|>`). Model output is hostile input: malformed calls are skipped, never panic,
//! and every loop makes progress.

use serde_json::{Map, Number, Value};

const Q: &str = "<|\"|>";
const MAX_DEPTH: usize = 32;

#[derive(Debug, Clone, PartialEq)]
pub struct ToolCall {
    pub name: String,
    pub arguments: Value,
}

struct Cursor<'a> {
    s: &'a str,
    i: usize,
}

impl Cursor<'_> {
    fn rest(&self) -> &str {
        self.s.get(self.i..).unwrap_or("")
    }
    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }
    fn bump(&mut self) {
        if let Some(c) = self.peek() {
            self.i += c.len_utf8();
        }
    }
    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(' ' | '\t' | '\r' | '\n')) {
            self.bump();
        }
    }
    fn starts(&self, p: &str) -> bool {
        self.rest().starts_with(p)
    }
}

fn parse_value(c: &mut Cursor, depth: usize) -> Result<Value, ()> {
    if depth > MAX_DEPTH {
        return Err(());
    }
    c.skip_ws();
    if c.starts(Q) {
        c.i += Q.len();
        let end = c.rest().find(Q).ok_or(())?;
        let v = c.rest().get(..end).unwrap_or("").to_string();
        c.i += end + Q.len();
        return Ok(Value::String(v));
    }
    match c.peek() {
        Some('"') => {
            c.bump();
            let mut buf = String::new();
            loop {
                match c.peek() {
                    None => break,
                    Some('"') => {
                        c.bump();
                        break;
                    }
                    Some('\\') => {
                        c.bump();
                        match c.peek() {
                            Some('n') => buf.push('\n'),
                            Some('t') => buf.push('\t'),
                            Some(x) => buf.push(x),
                            None => buf.push('\\'),
                        }
                        c.bump();
                    }
                    Some(x) => {
                        buf.push(x);
                        c.bump();
                    }
                }
            }
            Ok(Value::String(buf))
        }
        Some('{') => {
            c.bump();
            parse_object(c, depth + 1)
        }
        Some('[') => {
            c.bump();
            let mut out = Vec::new();
            loop {
                c.skip_ws();
                match c.peek() {
                    None => return Err(()),
                    Some(']') => {
                        c.bump();
                        return Ok(Value::Array(out));
                    }
                    _ => {}
                }
                out.push(parse_value(c, depth + 1)?);
                c.skip_ws();
                match c.peek() {
                    Some(',') => c.bump(),
                    Some(']') | None => {}
                    Some(_) => return Err(()),
                }
            }
        }
        _ => {
            let start = c.i;
            while !matches!(c.peek(), None | Some(',' | '}' | ']')) {
                c.bump();
            }
            if c.i == start {
                return Err(());
            }
            let raw = c.s.get(start..c.i).unwrap_or("").trim();
            Ok(scalar(raw))
        }
    }
}

fn scalar(raw: &str) -> Value {
    match raw {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        "null" => return Value::Null,
        _ => {}
    }
    let int_like = !raw.is_empty() && raw.strip_prefix('-').unwrap_or(raw).chars().all(|ch| ch.is_ascii_digit()) && raw != "-";
    if int_like && let Ok(i) = raw.parse::<i64>() {
        return Value::Number(i.into());
    }
    if let Ok(f) = raw.parse::<f64>()
        && let Some(n) = Number::from_f64(f)
    {
        return Value::Number(n);
    }
    Value::String(raw.to_string())
}

fn parse_object(c: &mut Cursor, depth: usize) -> Result<Value, ()> {
    if depth > MAX_DEPTH {
        return Err(());
    }
    let mut out = Map::new();
    loop {
        c.skip_ws();
        let key = match c.peek() {
            None => return Err(()),
            Some('}') => {
                c.bump();
                return Ok(Value::Object(out));
            }
            _ if c.starts(Q) => {
                c.i += Q.len();
                let end = c.rest().find(Q).ok_or(())?;
                let k = c.rest().get(..end).unwrap_or("").to_string();
                c.i += end + Q.len();
                k
            }
            Some('"') => {
                c.bump();
                let end = c.rest().find('"').ok_or(())?;
                let k = c.rest().get(..end).unwrap_or("").to_string();
                c.i += end + 1;
                k
            }
            _ => {
                let end = c.rest().find(':').ok_or(())?;
                let k = c.rest().get(..end).unwrap_or("").trim().to_string();
                c.i += end;
                k
            }
        };
        c.skip_ws();
        if c.peek() == Some(':') {
            c.bump();
        }
        let v = parse_value(c, depth)?;
        out.insert(key, v);
        c.skip_ws();
        match c.peek() {
            Some(',') => c.bump(),
            Some('}') | None => {}
            Some(_) => return Err(()),
        }
    }
}

/// Every well-formed native tool call in `text`, in order.
pub fn parse_tool_calls(text: &str) -> Vec<ToolCall> {
    let Ok(re) = regex::Regex::new(r"<\|tool_call>\s*call:\s*([A-Za-z_][\w\-.]*)\s*\{") else { return Vec::new() };
    re.captures_iter(text)
        .filter_map(|cap| {
            let m = cap.get(0)?;
            let mut c = Cursor { s: text, i: m.end() };
            let args = parse_object(&mut c, 0).ok()?;
            Some(ToolCall { name: cap.get(1)?.as_str().to_string(), arguments: args })
        })
        .collect()
}

const SPECIAL_STRIP: [&str; 5] = ["<eos>", "<turn|>", "<end_of_turn>", "<|tool_response>", "<bos>"];

fn clean_final(t: &str) -> String {
    let mut s = t.to_string();
    for sp in SPECIAL_STRIP {
        s = s.replace(sp, "");
    }
    s.trim().to_string()
}

/// (thought, final text, closed): `closed` is false when a thinking channel was opened but never closed (cut off).
pub fn split_thinking(t: &str) -> (String, String, bool) {
    if t.contains("<|channel>") || t.contains("<channel|>") {
        if let Some((head, fin)) = t.rsplit_once("<channel|>") {
            let thought = head.split_once("<|channel>").map(|(_, b)| b).unwrap_or(head);
            let thought = thought.strip_prefix("thought").unwrap_or(thought);
            return (thought.trim().to_string(), clean_final(fin), true);
        }
        let thought = t.split_once("<|channel>").map(|(_, b)| b).unwrap_or(t);
        let thought = thought.strip_prefix("thought").unwrap_or(thought);
        return (thought.trim().to_string(), String::new(), false);
    }
    (String::new(), clean_final(t), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hostile_inputs_never_panic_or_hang() {
        let deep = format!("<|tool_call>call:x{}", "{a:".repeat(5000));
        for t in
            ["<|tool_call>call:x{", "<|tool_call>call:x{a:[", "<|tool_call>call:x{\"a", "<|tool_call>call:x{a:\"\\", &deep, "<|tool_call>call:é{a:1}"]
        {
            let _ = parse_tool_calls(t);
            let _ = split_thinking(t);
        }
    }
}
