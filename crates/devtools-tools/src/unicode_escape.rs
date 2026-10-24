//! Text ↔ Unicode escapes (\uXXXX / U+XXXX).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::UNICODE_ESC,
        name: "Unicode escape",
        category: Category::Encoding,
        summary: "文本与 \\uXXXX / U+XXXX 互转；补充「文本二进制」码点视图。",
        keywords: &["unicode", "escape", "\\u", "码点"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    if looks_like_escapes(t) {
        let text = decode_escapes(t)?;
        return Ok(outputs(&text));
    }
    Ok(outputs(t))
}

fn outputs(text: &str) -> Vec<(&'static str, String)> {
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
        let rows = convert_all("你").unwrap();
        assert!(rows[1].1.contains("\\u"));
    }
    #[test]
    fn decode_js() {
        let rows = convert_all("\\u4F60").unwrap();
        assert_eq!(rows[0].1, "你");
    }
}
