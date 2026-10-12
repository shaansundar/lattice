//! Python-compatible text formatting.
//!
//! The model was trained on observations rendered by a Python reference harness, so the in-app harness must produce
//! the same bytes: Python's `repr` of floats (shortest round-trip, scientific outside 1e-4..1e16), `round(x, 6)`,
//! `str()` of JSON values, and `json.dumps` with its default `ensure_ascii` escaping.

use serde_json::Value;

/// Python `repr(float)`.
pub fn float_repr(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if f == 0.0 {
        return if f.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    // Rust's `{:e}` prints the shortest round-trip digits, like Python's repr: "-1.5e-5", "4.82e4", "1e16".
    let s = format!("{f:e}");
    let Some((mant, exp)) = s.split_once('e') else { return s };
    let Ok(exp) = exp.parse::<i32>() else { return s };
    let neg = mant.starts_with('-');
    let digits: String = mant.chars().filter(char::is_ascii_digit).collect();
    let sign = if neg { "-" } else { "" };
    if !(-4..16).contains(&exp) {
        let m = match digits.split_at_checked(1) {
            Some((a, b)) if !b.is_empty() => format!("{a}.{b}"),
            _ => digits.clone(),
        };
        return format!("{sign}{m}e{}{:02}", if exp < 0 { '-' } else { '+' }, exp.unsigned_abs());
    }
    if exp >= 0 {
        let point = (exp as usize) + 1;
        if digits.len() <= point {
            format!("{sign}{digits}{}.0", "0".repeat(point - digits.len()))
        } else {
            let (a, b) = digits.split_at(point);
            format!("{sign}{a}.{b}")
        }
    } else {
        format!("{sign}0.{}{digits}", "0".repeat(exp.unsigned_abs() as usize - 1))
    }
}

/// Python `round(f, 6)` (correctly rounded, like formatting with 6 decimals).
pub fn round6(f: f64) -> f64 {
    format!("{f:.6}").parse().unwrap_or(f)
}

/// Python `str()` of a JSON value parsed by Python's `json` module.
pub fn py_str(v: &Value) -> String {
    match v {
        Value::Null => "None".into(),
        Value::Bool(b) => if *b { "True" } else { "False" }.into(),
        Value::Number(n) => number_str(n),
        Value::String(s) => s.clone(),
        Value::Array(_) | Value::Object(_) => py_repr(v),
    }
}

fn number_str(n: &serde_json::Number) -> String {
    if let Some(i) = n.as_i64() {
        return i.to_string();
    }
    if let Some(u) = n.as_u64() {
        return u.to_string();
    }
    float_repr(n.as_f64().unwrap_or(0.0))
}

/// Python `repr()` of a JSON value (strings single-quoted unless they contain a quote and no double quote).
pub fn py_repr(v: &Value) -> String {
    match v {
        Value::String(s) => {
            let q = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
            let mut out = String::from(q);
            for c in s.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c == q => {
                        out.push('\\');
                        out.push(c);
                    }
                    c if (c as u32) < 0x20 || c as u32 == 0x7f => out.push_str(&format!("\\x{:02x}", c as u32)),
                    c => out.push(c),
                }
            }
            out.push(q);
            out
        }
        Value::Array(a) => format!("[{}]", a.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Object(o) => {
            format!("{{{}}}", o.iter().map(|(k, v)| format!("{}: {}", py_repr(&Value::String(k.clone())), py_repr(v))).collect::<Vec<_>>().join(", "))
        }
        other => py_str(other),
    }
}

/// Python `json.dumps(v)` (`ensure_ascii=True`); `compact` uses separators (",", ":"); keys are sorted
/// (serde_json's map is ordered by key, which matches `sort_keys=True`).
pub fn json_dumps(v: &Value, compact: bool) -> String {
    let mut out = String::new();
    write_json(&mut out, v, compact);
    out
}

fn write_json(out: &mut String, v: &Value, compact: bool) {
    let (item, kv) = if compact { (",", ":") } else { (", ", ": ") };
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => match (n.as_i64(), n.as_u64()) {
            (Some(i), _) => out.push_str(&i.to_string()),
            (_, Some(u)) => out.push_str(&u.to_string()),
            _ => {
                let f = n.as_f64().unwrap_or(0.0);
                out.push_str(&if f.is_nan() {
                    "NaN".into()
                } else if f.is_infinite() {
                    if f > 0.0 { "Infinity".into() } else { "-Infinity".into() }
                } else {
                    float_repr(f)
                });
            }
        },
        Value::String(s) => write_json_str(out, s),
        Value::Array(a) => {
            out.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    out.push_str(item);
                }
                write_json(out, x, compact);
            }
            out.push(']');
        }
        Value::Object(o) => {
            out.push('{');
            for (i, (k, x)) in o.iter().enumerate() {
                if i > 0 {
                    out.push_str(item);
                }
                write_json_str(out, k);
                out.push_str(kv);
                write_json(out, x, compact);
            }
            out.push('}');
        }
    }
}

fn write_json_str(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7f => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Truncate to `max` characters with a trailing "…" (Python slicing by code points).
pub fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut out: String = s.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn float_repr_matches_python() {
        for (f, want) in [
            (0.28361, "0.28361"),
            (87091.428571, "87091.428571"),
            (5.0, "5.0"),
            (1e16, "1e+16"),
            (1e15, "1000000000000000.0"),
            (1e-5, "1e-05"),
            (1.5e-5, "1.5e-05"),
            (0.0001, "0.0001"),
            (-2.5, "-2.5"),
            (0.1 + 0.2, "0.30000000000000004"),
            (123456789.0, "123456789.0"),
            (-1.2345e20, "-1.2345e+20"),
        ] {
            assert_eq!(float_repr(f), want, "{f}");
        }
        assert_eq!(float_repr(round6(0.000001234)), "1e-06");
    }

    #[test]
    fn json_dumps_matches_python_ensure_ascii() {
        assert_eq!(json_dumps(&json!("\u{20ac} \"x\"\n"), false), "\"\\u20ac \\\"x\\\"\\n\"");
        assert_eq!(json_dumps(&json!("\u{1F600}"), false), "\"\\ud83d\\ude00\"");
        assert_eq!(json_dumps(&json!({"b": 1, "a": [1.5, null, true]}), true), r#"{"a":[1.5,null,true],"b":1}"#);
        assert_eq!(py_str(&json!(null)), "None");
        assert_eq!(py_repr(&json!(["a", 1])), "['a', 1]");
        assert_eq!(truncate("abcdef", 4), "abc…");
    }
}
