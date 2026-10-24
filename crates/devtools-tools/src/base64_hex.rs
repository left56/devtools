//! Hex ↔ Base64 payload converter (paste-based "file" substitute).

use base64::Engine;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::BASE64_HEX,
        name: "Base64 ↔ Hex",
        category: Category::Encoding,
        summary: "粘贴十六进制或 Base64，互转原始字节表示（无文件选择器）。",
        keywords: &["base64", "hex", "file", "二进制", "载荷"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("粘贴 hex（可含空格）或 Base64".into());
    }
    let cleaned_ws: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    // Prefer hex if only hex digits (and optional 0x)
    let hex_body = cleaned_ws
        .strip_prefix("0x")
        .or_else(|| cleaned_ws.strip_prefix("0X"))
        .unwrap_or(&cleaned_ws);
    if hex_body.len() >= 2
        && hex_body.len().is_multiple_of(2)
        && hex_body.bytes().all(|b| b.is_ascii_hexdigit())
        && !looks_like_mostly_base64(&cleaned_ws)
    {
        let bytes = decode_hex(hex_body)?;
        return Ok(rows_from_bytes(&bytes, "hex"));
    }
    // else base64
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&cleaned_ws)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&cleaned_ws))
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(&cleaned_ws))
        .map_err(|e| format!("既不是合法 hex 也不是 Base64：{e}"))?;
    Ok(rows_from_bytes(&bytes, "base64"))
}

fn looks_like_mostly_base64(s: &str) -> bool {
    s.contains('+') || s.contains('/') || s.contains('-') || s.contains('_') || s.ends_with('=')
}

fn decode_hex(s: &str) -> Result<Vec<u8>, String> {
    let mut out = Vec::with_capacity(s.len() / 2);
    let b = s.as_bytes();
    let mut i = 0;
    while i + 1 < b.len() {
        let h = std::str::from_utf8(&b[i..i + 2]).unwrap();
        out.push(u8::from_str_radix(h, 16).map_err(|_| format!("非法 hex `{h}`"))?);
        i += 2;
    }
    Ok(out)
}

fn rows_from_bytes(bytes: &[u8], detected: &str) -> Vec<(&'static str, String)> {
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let hex_spaced: String = bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    let utf8 = match std::str::from_utf8(bytes) {
        Ok(s) if s.chars().all(|c| !c.is_control() || c == '\n' || c == '\t') => s.to_string(),
        _ => "（非可打印 UTF-8）".into(),
    };
    vec![
        ("检测为", detected.into()),
        ("字节数", bytes.len().to_string()),
        ("Hex", hex),
        ("Hex 空格", hex_spaced),
        ("Base64", b64),
        ("UTF-8 预览", utf8),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hex_to_b64() {
        let rows = convert_all("68656c6c6f").unwrap();
        assert_eq!(rows.iter().find(|r| r.0 == "Base64").unwrap().1, "aGVsbG8=");
    }
    #[test]
    fn b64_to_hex() {
        let rows = convert_all("aGVsbG8=").unwrap();
        assert_eq!(rows.iter().find(|r| r.0 == "Hex").unwrap().1, "68656c6c6f");
    }
}
