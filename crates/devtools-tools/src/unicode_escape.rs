//! Text ↔ Unicode escapes (\uXXXX / U+XXXX).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::UNICODE_ESC,
        name: "Unicode escape",
        category: Category::Encoding,
        summary: "文本与 \\uXXXX / U+XXXX 互转。模式：转义 | 反转义。",
        keywords: &["unicode", "escape", "\\u", "码点"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnicodeMode {
    Escape,
    Unescape,
}

impl UnicodeMode {
    pub const ALL: [UnicodeMode; 2] = [UnicodeMode::Escape, UnicodeMode::Unescape];

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

/// Prefer unescape when input looks like escapes.
pub fn suggest_mode(input: &str) -> UnicodeMode {
    if looks_like_escapes(input.trim()) {
        UnicodeMode::Unescape
    } else {
        UnicodeMode::Escape
    }
}

pub fn convert_mode(input: &str, mode: UnicodeMode) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    match mode {
        UnicodeMode::Escape => Ok(escape_rows(t)),
        UnicodeMode::Unescape => {
            let text = decode_escapes(t)?;
            Ok(vec![("文本", text)])
        }
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {

    convert_mode(input, suggest_mode(input))
}

fn escape_rows(text: &str) -> Vec<(&'static str, String)> {
    let js: String = text
        .chars()
        .map(|c| {
            let n = c as u32;
            if n <= 0xFFFF {
                format!("\\u{n:04X}")
            } else {
                let n = n - 0x10000;
                let hi = 0xD800 + (n >> 10);
                let lo = 0xDC00 + (n & 0x3FF);
                format!("\\u{hi:04X}\\u{lo:04X}")
            }
        })
        .collect();
    let plus: String = text
        .chars()
        .map(|c| format!("U+{:04X}", c as u32))
        .collect::<Vec<_>>()
        .join(" ");
    let html: String = text
        .chars()
        .map(|c| format!("&#x{:X};", c as u32))
        .collect();
    vec![
        ("文本", text.to_string()),
        ("\\u 转义", js),
        ("U+ 码点", plus),
        ("HTML 实体", html),
    ]
}

fn looks_like_escapes(s: &str) -> bool {
    s.contains("\\u") || s.contains("U+") || s.contains("&#x") || s.contains("&#X")
}

fn decode_escapes(s: &str) -> Result<String, String> {
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 5 < bytes.len() && bytes[i + 1] == b'u' {
            let hex = std::str::from_utf8(&bytes[i + 2..i + 6]).map_err(|e| e.to_string())?;
            let cp = u32::from_str_radix(hex, 16).map_err(|_| "非法 \\u 十六进制".to_string())?;
            out.push(char::from_u32(cp).ok_or("非法码点")?);
            i += 6;
        } else if bytes[i] == b'U' && i + 1 < bytes.len() && bytes[i + 1] == b'+' {
            i += 2;
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_hexdigit() {
                i += 1;
            }
            if i == start {
                return Err("U+ 后缺少十六进制".into());
            }
            let hex = std::str::from_utf8(&bytes[start..i]).unwrap();
            let cp = u32::from_str_radix(hex, 16).map_err(|_| "非法 U+ 十六进制".to_string())?;
            out.push(char::from_u32(cp).ok_or("非法码点")?);
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
        } else if bytes[i] == b'&' && i + 3 < bytes.len() && bytes[i + 1] == b'#' {
            // &#xHH; or &#NN;
            i += 2;
            let hex_mode = bytes[i] == b'x' || bytes[i] == b'X';
            if hex_mode {
                i += 1;
            }
            let start = i;
            while i < bytes.len() && bytes[i] != b';' {
                i += 1;
            }
            let num = std::str::from_utf8(&bytes[start..i]).unwrap();
            let cp = if hex_mode {
                u32::from_str_radix(num, 16)
            } else {
                num.parse()
            }
            .map_err(|_| "非法 HTML 实体".to_string())?;
            out.push(char::from_u32(cp).ok_or("非法码点")?);
            if i < bytes.len() && bytes[i] == b';' {
                i += 1;
            }
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encode_chinese() {
        let rows = convert_mode("你", UnicodeMode::Escape).unwrap();
        assert!(rows.iter().any(|r| r.1.contains("4F60") || r.1.contains("4f60")));
        assert_eq!(suggest_mode("你"), UnicodeMode::Escape);
    }
    #[test]
    fn decode_js() {
        let rows = convert_mode("\\u4F60", UnicodeMode::Unescape).unwrap();
        assert_eq!(rows[0].1, "你");
        assert_eq!(suggest_mode("\\u4F60"), UnicodeMode::Unescape);
    }
}
