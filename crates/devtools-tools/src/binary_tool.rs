//! Text ↔ binary / hex / code points.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::BINARY,
        name: "Text binary",
        category: Category::Encoding,
        summary: "文本与二进制、十六进制、Unicode 码点互转。自动识别输入。",
        keywords: &["binary", "hex", "unicode", "二进制", "码点"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    let text = if looks_like_binary(t) {
        decode_binary(t)?
    } else if looks_like_hex(t) {
        decode_hex(t)?
    } else {
        t.to_string()
    };
    let bytes = text.as_bytes();
    let binary = bytes
        .iter()
        .map(|b| format!("{b:08b}"))
        .collect::<Vec<_>>()
        .join(" ");
    let hex = bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    let codepoints = text
        .chars()
        .map(|c| format!("U+{:04X}", c as u32))
        .collect::<Vec<_>>()
        .join(" ");
    Ok(vec![
        ("文本", text),
        ("二进制", binary),
        ("十六进制", hex),
        ("Unicode 码点", codepoints),
    ])
}

fn looks_like_binary(s: &str) -> bool {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    !cleaned.is_empty()
        && cleaned.len().is_multiple_of(8)
        && cleaned.bytes().all(|b| b == b'0' || b == b'1')
        && cleaned.len() >= 8
}

fn looks_like_hex(s: &str) -> bool {
    let cleaned: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    let body = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
        .unwrap_or(&cleaned);
    body.len() >= 2
        && body.len().is_multiple_of(2)
        && body.bytes().all(|b| b.is_ascii_hexdigit())
        // Avoid treating short words as hex
        && body.bytes().any(|b| b.is_ascii_digit())
}

fn decode_binary(s: &str) -> Result<String, String> {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if !cleaned.len().is_multiple_of(8) {
        return Err("二进制长度须为 8 的倍数".into());
    }
    let mut bytes = Vec::new();
    for chunk in cleaned.as_bytes().chunks(8) {
        let bitstr = std::str::from_utf8(chunk).unwrap();
        let b = u8::from_str_radix(bitstr, 2).map_err(|_| format!("无效二进制 `{bitstr}`"))?;
        bytes.push(b);
    }
    String::from_utf8(bytes).map_err(|_| "二进制不是合法 UTF-8".into())
}

fn decode_hex(s: &str) -> Result<String, String> {
    let cleaned: String = s
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .collect();
    let body = cleaned
        .strip_prefix("0x")
        .or_else(|| cleaned.strip_prefix("0X"))
        .unwrap_or(&cleaned);
    if !body.len().is_multiple_of(2) {
        return Err("十六进制长度须为偶数".into());
    }
    let mut bytes = Vec::new();
    for chunk in body.as_bytes().chunks(2) {
        let hs = std::str::from_utf8(chunk).unwrap();
        let b = u8::from_str_radix(hs, 16).map_err(|_| format!("无效十六进制 `{hs}`"))?;
        bytes.push(b);
    }
    String::from_utf8(bytes).map_err(|_| "十六进制不是合法 UTF-8".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_to_all() {
        let rows = convert_all("Hi").unwrap();
        assert_eq!(rows[0].1, "Hi");
        assert!(rows[1].1.contains("01001000"));
        assert_eq!(rows[2].1, "48 69");
    }

    #[test]
    fn binary_round() {
        let rows = convert_all("01001000 01101001").unwrap();
        assert_eq!(rows[0].1, "Hi");
    }
}
