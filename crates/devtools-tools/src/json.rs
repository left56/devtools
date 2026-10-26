//! Strict JSON format / minify, plus optional indent and format knobs.

use std::fmt;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use serde_json::{Map, Value};

mod tree;
pub use tree::{JsonDocument, JsonKind, JsonNode};

/// JSON parse failure. Positions match `serde_json`: 1-based line and byte
/// column; the column can be zero for empty input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "第 {} 行、第 {} 列：{}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for JsonError {}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::JSON,
        name: "JSON",
        category: Category::Data,
        summary: "格式化 / 压缩 JSON。可选缩进、尾随逗号与注释、自动修复、排序键、保留转义与大数。",
        keywords: &[
            "json",
            "format",
            "pretty",
            "minify",
            "格式化",
            "压缩",
            "tree",
            "path",
            "jq",
            "树",
            "路径",
            "json5",
            "indent",
        ],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndentStyle {
    Spaces2,
    Spaces4,
    Tab,
    Minified,
}

impl IndentStyle {
    pub const ALL: [Self; 4] = [
        Self::Spaces2,
        Self::Spaces4,
        Self::Tab,
        Self::Minified,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Spaces2 => "2 空格",
            Self::Spaces4 => "4 空格",
            Self::Tab => "Tab",
            Self::Minified => "压缩",
        }
    }

    pub fn default_style() -> Self {
        Self::Spaces2
    }

    fn indent_bytes(self) -> Option<&'static [u8]> {
        match self {
            Self::Spaces2 => Some(b"  "),
            Self::Spaces4 => Some(b"    "),
            Self::Tab => Some(b"\t"),
            Self::Minified => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FormatOptions {
    pub indent: IndentStyle,
    /// Allow trailing commas and // /* */ comments (JSON5 subset → strict JSON out).
    pub allow_comments_commas: bool,
    /// Best-effort repair before parse (Python literals, trailing commas, etc.).
    pub auto_repair: bool,
    /// Sort object keys lexicographically (recursive).
    pub sort_keys: bool,
    /// Preserve `\uXXXX` escapes and raw number literals. Only when sort+repair off.
    pub preserve_raw: bool,
}

impl FormatOptions {
    pub fn default_options() -> Self {
        Self {
            indent: IndentStyle::default_style(),
            allow_comments_commas: false,
            auto_repair: false,
            sort_keys: false,
            preserve_raw: false,
        }
    }

    /// Preserve only applies when sort_keys and auto_repair are both off.
    pub fn preserve_active(self) -> bool {
        self.preserve_raw && !self.sort_keys && !self.auto_repair
    }
}

/// Pretty-print strict JSON with two-space indentation.
pub fn prettify(input: &str) -> Result<String, JsonError> {
    format_with_options(input, FormatOptions::default_options())
}

/// Emit compact strict JSON. Key order follows the input.
pub fn minify(input: &str) -> Result<String, JsonError> {
    let mut opts = FormatOptions::default_options();
    opts.indent = IndentStyle::Minified;
    format_with_options(input, opts)
}

pub fn format_with_options(input: &str, opts: FormatOptions) -> Result<String, JsonError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(JsonError {
            message: "输入为空".into(),
            line: 1,
            column: 0,
        });
    }

    if opts.preserve_active() {
        return reindent_preserving(input, opts);
    }

    let prepared = if opts.auto_repair {
        repair_light(input)
    } else {
        input.to_string()
    };

    let mut value = parse_flexible(&prepared, opts.allow_comments_commas)?;
    if opts.sort_keys {
        value = sort_value(value);
    }
    emit_value(&value, opts.indent)
}

fn parse(input: &str) -> Result<Value, JsonError> {
    serde_json::from_str(input).map_err(map_serde_err)
}

fn parse_flexible(input: &str, allow_json5: bool) -> Result<Value, JsonError> {
    match serde_json::from_str::<Value>(input) {
        Ok(v) => Ok(v),
        Err(_strict_err) if allow_json5 => json5::from_str(input).map_err(|e| JsonError {
            message: format!("JSON5 解析失败：{e}"),
            line: 1,
            column: 1,
        }),
        Err(e) => Err(map_serde_err(e)),
    }
}

fn map_serde_err(error: serde_json::Error) -> JsonError {
    JsonError {
        message: error.to_string(),
        line: error.line(),
        column: error.column(),
    }
}

fn emit_value(value: &Value, indent: IndentStyle) -> Result<String, JsonError> {
    match indent.indent_bytes() {
        None => Ok(serde_json::to_string(value).expect("serialize Value")),
        Some(indent_bytes) => {
            let mut buf = Vec::new();
            let formatter = serde_json::ser::PrettyFormatter::with_indent(indent_bytes);
            let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
            serde::Serialize::serialize(value, &mut ser).map_err(|e| JsonError {
                message: e.to_string(),
                line: 1,
                column: 1,
            })?;
            Ok(String::from_utf8(buf).expect("serde_json writes utf8"))
        }
    }
}


fn sort_value(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut sorted = Map::new();
            for (k, v) in entries {
                sorted.insert(k, sort_value(v));
            }
            Value::Object(sorted)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(sort_value).collect()),
        other => other,
    }
}

/// Lightweight repair: strip comments, trailing commas, Python constants,
/// and single-quoted strings → double-quoted JSON strings.
fn repair_light(input: &str) -> String {
    let mut s = strip_comments(input);
    s = s
        .replace(": True", ": true")
        .replace(": False", ": false")
        .replace(": None", ": null")
        .replace(":True", ": true")
        .replace(":False", ": false")
        .replace(":None", ": null");
    // Also bare True/False/None as values in arrays
    s = replace_word(&s, "True", "true");
    s = replace_word(&s, "False", "false");
    s = replace_word(&s, "None", "null");
    s = single_quotes_to_double(&s);
    strip_trailing_commas(&s)
}

/// Convert `'…'` string literals to `"…"` outside of double-quoted strings.
fn single_quotes_to_double(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let mut in_double = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_double {
            out.push(b as char);
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_double = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_double = true;
            out.push('"');
            i += 1;
            continue;
        }
        if b == b'\'' {
            out.push('"');
            i += 1;
            while i < bytes.len() {
                let c = bytes[i];
                if c == b'\\' && i + 1 < bytes.len() {
                    let n = bytes[i + 1];
                    if n == b'\'' {
                        out.push('\'');
                        i += 2;
                        continue;
                    }
                    if n == b'\\' {
                        out.push_str("\\\\");
                        i += 2;
                        continue;
                    }
                    out.push('\\');
                    out.push(n as char);
                    i += 2;
                    continue;
                }
                if c == b'\'' {
                    out.push('"');
                    i += 1;
                    break;
                }
                if c == b'"' {
                    out.push_str("\\\"");
                    i += 1;
                    continue;
                }
                out.push(c as char);
                i += 1;
            }
            continue;
        }
        out.push(b as char);
        i += 1;
    }
    out
}

fn replace_word(input: &str, from: &str, to: &str) -> String {
    // Avoid replacing inside strings: crude scan.
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let mut in_string = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            out.push(b as char);
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            out.push('"');
            i += 1;
            continue;
        }
        if input[i..].starts_with(from) {
            let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric();
            let after = i + from.len();
            let after_ok = after >= bytes.len() || !bytes[after].is_ascii_alphanumeric();
            if before_ok && after_ok {
                out.push_str(to);
                i = after;
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    out
}

fn strip_comments(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let mut in_string = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            out.push(b as char);
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            out.push('"');
            i += 1;
            continue;
        }
        if b == b'/' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'/' {
                i += 2;
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if bytes[i + 1] == b'*' {
                i += 2;
                while i + 1 < bytes.len() && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i = (i + 2).min(bytes.len());
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    out
}

fn strip_trailing_commas(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let mut in_string = false;
    let mut escape = false;
    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            out.push(b as char);
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_string = true;
            out.push('"');
            i += 1;
            continue;
        }
        if b == b',' {
            let mut j = i + 1;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j < bytes.len() && (bytes[j] == b'}' || bytes[j] == b']') {
                i += 1;
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    out
}

/// Re-indent without re-serializing typed values (keeps `\u` and big number text).
fn reindent_preserving(input: &str, opts: FormatOptions) -> Result<String, JsonError> {
    let source = if opts.allow_comments_commas {
        let stripped = strip_comments(input);
        strip_trailing_commas(&stripped)
    } else {
        // Still must be valid JSON structurally — validate first.
        let _ = parse(input)?;
        input.to_string()
    };
    // Validate after comment/comma strip when json5-ish.
    if opts.allow_comments_commas {
        let _ = parse_flexible(&source, true)?;
    }
    Ok(reindent_tokens(&source, opts.indent))
}

fn reindent_tokens(input: &str, indent: IndentStyle) -> String {
    let indent_unit = match indent {
        IndentStyle::Spaces2 => "  ",
        IndentStyle::Spaces4 => "    ",
        IndentStyle::Tab => "\t",
        IndentStyle::Minified => "",
    };
    let minify = matches!(indent, IndentStyle::Minified);
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let mut depth: usize = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut pending_space = false;

    let write_indent = |out: &mut String, depth: usize| {
        if minify {
            return;
        }
        out.push('\n');
        for _ in 0..depth {
            out.push_str(indent_unit);
        }
    };

    while i < bytes.len() {
        let b = bytes[i];
        if in_string {
            out.push(b as char);
            if escape {
                escape = false;
            } else if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_string = false;
            }
            i += 1;
            continue;
        }
        match b {
            b'"' => {
                if pending_space && !minify {
                    out.push(' ');
                }
                pending_space = false;
                in_string = true;
                out.push('"');
                i += 1;
            }
            b'{' | b'[' => {
                pending_space = false;
                out.push(b as char);
                depth += 1;
                // peek skip whitespace
                let mut j = i + 1;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] != b'}' && bytes[j] != b']' {
                    write_indent(&mut out, depth);
                }
                i += 1;
            }
            b'}' | b']' => {
                pending_space = false;
                depth = depth.saturating_sub(1);
                if !minify {
                    // if previous non-ws isn't opener on same empty container
                    let trimmed = out.trim_end();
                    let empty = trimmed.ends_with('{') || trimmed.ends_with('[');
                    if !empty {
                        write_indent(&mut out, depth);
                    }
                }
                out.push(b as char);
                i += 1;
            }
            b',' => {
                pending_space = false;
                out.push(',');
                write_indent(&mut out, depth);
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
            }
            b':' => {
                out.push(':');
                pending_space = !minify;
                i += 1;
                while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
            }
            b if b.is_ascii_whitespace() => {
                i += 1;
            }
            _ => {
                if pending_space && !minify {
                    out.push(' ');
                }
                pending_space = false;
                // copy literal (number / true / false / null)
                while i < bytes.len() {
                    let c = bytes[i];
                    if c.is_ascii_whitespace()
                        || matches!(c, b',' | b'}' | b']' | b':' | b'{' | b'[')
                    {
                        break;
                    }
                    out.push(c as char);
                    i += 1;
                }
            }
        }
    }
    if !minify && !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prettify_keeps_key_order_and_minify_round_trips() {
        let input = r#"{"b":1,"a":2}"#;
        assert_eq!(prettify(input).unwrap(), "{\n  \"b\": 1,\n  \"a\": 2\n}");
        assert_eq!(minify(input).unwrap(), input);
        assert_eq!(minify(&prettify(input).unwrap()).unwrap(), input);
    }

    #[test]
    fn unicode_strings_round_trip() {
        let input = r#"{"name":"你好"}"#;
        assert_eq!(minify(input).unwrap(), input);
    }

    #[test]
    fn trailing_comma_is_rejected_by_default() {
        let error = prettify(r#"{"a":1,}"#).unwrap_err();
        assert!(error.line >= 1);
    }

    #[test]
    fn empty_input_is_rejected() {
        assert!(minify("").is_err());
        assert!(minify("   ").is_err());
    }

    #[test]
    fn indent_four_spaces_and_tab() {
        let input = r#"{"a":1}"#;
        let mut opts = FormatOptions::default_options();
        opts.indent = IndentStyle::Spaces4;
        let out = format_with_options(input, opts).unwrap();
        assert!(out.contains("\n    \"a\":"));
        opts.indent = IndentStyle::Tab;
        let out = format_with_options(input, opts).unwrap();
        assert!(out.contains("\n\t\"a\":"));
    }

    #[test]
    fn sort_keys() {
        let mut opts = FormatOptions::default_options();
        opts.sort_keys = true;
        opts.indent = IndentStyle::Minified;
        assert_eq!(
            format_with_options(r#"{"b":1,"a":2}"#, opts).unwrap(),
            r#"{"a":2,"b":1}"#
        );
    }

    #[test]
    fn json5_comments_and_trailing_comma() {
        let mut opts = FormatOptions::default_options();
        opts.allow_comments_commas = true;
        opts.indent = IndentStyle::Minified;
        let out = format_with_options("{a: 1, /* c */}", opts).unwrap();
        assert!(out.contains("\"a\""));
        assert!(out.contains('1'));
    }

    #[test]
    fn auto_repair_python_and_trailing() {
        let mut opts = FormatOptions::default_options();
        opts.auto_repair = true;
        opts.indent = IndentStyle::Minified;
        assert_eq!(
            format_with_options("{'a': True,}", opts).unwrap(),
            r#"{"a":true}"#
        );
        assert_eq!(
            format_with_options(r#"{"a": True,}"#, opts).unwrap(),
            r#"{"a":true}"#
        );
    }

    #[test]
    fn preserve_keeps_unicode_escape_and_big_number() {
        let mut opts = FormatOptions::default_options();
        opts.preserve_raw = true;
        opts.indent = IndentStyle::Spaces2;
        // Larger than u64 so serde_json would use f64 without preserve.
        let input = r#"{"n": 123456789012345678901234567890, "s": "\u00e2"}"#;
        let out = format_with_options(input, opts).unwrap();
        assert!(out.contains("123456789012345678901234567890"), "{out}");
        assert!(out.contains(r"\u00e2"), "{out}");
        opts.preserve_raw = false;
        let out2 = format_with_options(input, opts).unwrap();
        assert!(
            !out2.contains("123456789012345678901234567890")
                || out2.contains('e')
                || out2.contains('E'),
            "{out2}"
        );
    }
}
