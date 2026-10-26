//! HTML entity escape and unescape for the five XML/HTML special characters,
//! plus numeric character references on unescape.

use std::fmt;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HtmlEscapeError {
    pub message: String,
    /// Byte index into the input where parsing failed, when known.
    pub offset: Option<usize>,
}

impl fmt::Display for HtmlEscapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.offset {
            Some(offset) => write!(f, "偏移 {}：{}", offset, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for HtmlEscapeError {}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::HTML_ESCAPE,
        name: "HTML Escape",
        category: Category::Encoding,
        summary: "HTML 实体转义/反转义切换；含 & 实体时默认反转义。",
        keywords: &["html", "escape", "entity", "实体", "转义"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn unescape(input: &str) -> Result<String, HtmlEscapeError> {
    if input.is_empty() {
        return Err(HtmlEscapeError {
            message: "输入为空".into(),
            offset: None,
        });
    }
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'&' {
            let ch = input[i..].chars().next().expect("char boundary");
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let start = i;
        i += 1;
        let Some(end_rel) = bytes[i..].iter().position(|&b| b == b';') else {
            return Err(HtmlEscapeError {
                message: "实体缺少结尾分号".into(),
                offset: Some(start),
            });
        };
        let end = i + end_rel;
        let name = &input[i..end];
        i = end + 1;
        let ch = decode_entity(name).ok_or_else(|| HtmlEscapeError {
            message: format!("无法识别的实体 `&{name};`"),
            offset: Some(start),
        })?;
        out.push(ch);
    }
    Ok(out)
}

fn decode_entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ if name.starts_with('#') => decode_numeric(&name[1..]),
        _ => None,
    }
}

fn decode_numeric(body: &str) -> Option<char> {
    let (radix, digits) = if let Some(rest) = body
        .strip_prefix('x')
        .or_else(|| body.strip_prefix('X'))
    {
        (16, rest)
    } else {
        (10, body)
    };
    if digits.is_empty() {
        return None;
    }
    let value = u32::from_str_radix(digits, radix).ok()?;
    char::from_u32(value)
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HtmlMode {
    Escape,
    Unescape,
}

impl HtmlMode {
    pub const ALL: [HtmlMode; 2] = [HtmlMode::Escape, HtmlMode::Unescape];

    pub fn label(self) -> &'static str {
        match self {
            Self::Escape => "转义",
            Self::Unescape => "反转义",
        }
    }

    pub fn default_mode() -> Self {
        Self::Escape
    }
}

/// Prefer unescape when input contains entities that decode.
pub fn suggest_mode(input: &str) -> HtmlMode {
    let t = input.trim();
    if t.contains('&')
        && let Ok(u) = unescape(t)
        && u != t
    {
        return HtmlMode::Unescape;
    }
    HtmlMode::Escape
}

pub fn convert_mode(input: &str, mode: HtmlMode) -> Result<Vec<(&'static str, String)>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("输入为空".into());
    }
    match mode {
        HtmlMode::Escape => Ok(vec![("转义", escape(trimmed))]),
        HtmlMode::Unescape => {
            let u = unescape(trimmed).map_err(|e| e.to_string())?;
            Ok(vec![("反转义", u)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn convert_mode_entity() {
        let rows = convert_mode("&lt;a&gt;", HtmlMode::Unescape).unwrap();
        assert_eq!(rows[0].1, "<a>");
        assert_eq!(suggest_mode("&lt;a&gt;"), HtmlMode::Unescape);
        assert_eq!(suggest_mode("<a>"), HtmlMode::Escape);
    }

    #[test]
    fn empty_unescape_is_rejected() {
        assert!(unescape("").is_err());
    }

    #[test]
    fn escape_round_trips_specials() {
        let original = r#"<div class="a">&'b</div>"#;
        let escaped = escape(original);
        assert_eq!(
            escaped,
            "&lt;div class=&quot;a&quot;&gt;&amp;&#39;b&lt;/div&gt;"
        );
        assert_eq!(unescape(&escaped).unwrap(), original);
    }

    #[test]
    fn numeric_entities() {
        assert_eq!(unescape("&#65;&#x42;").unwrap(), "AB");
    }

    #[test]
    fn broken_entity_reports_offset() {
        let error = unescape("ok &bad").unwrap_err();
        assert!(error.message.contains("分号"));
        assert_eq!(error.offset, Some(3));
    }
}
