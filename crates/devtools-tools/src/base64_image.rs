//! Image ↔ Base64 / data-URL (offline). Path paste for encode; no camera.

use base64::Engine;
use std::path::Path;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base64ImageMode {
    Encode,
    Decode,
}

impl Base64ImageMode {
    pub const ALL: [Self; 2] = [Self::Encode, Self::Decode];
    pub fn label(self) -> &'static str {
        match self {
            Self::Encode => "编码",
            Self::Decode => "解码",
        }
    }
    pub fn default_mode() -> Self {
        Self::Encode
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::BASE64_IMAGE,
        name: "Base64 Image",
        category: Category::Graphics,
        summary: "图片路径 ↔ Base64 / data URL。离线；无相机/系统选图器，粘贴路径或 data URL。",
        keywords: &["base64", "image", "png", "jpeg", "data-url", "图片"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn suggest_mode(input: &str) -> Base64ImageMode {
    let t = input.trim();
    if t.starts_with("data:image/") || looks_b64(t) {
        Base64ImageMode::Decode
    } else {
        Base64ImageMode::Encode
    }
}

pub fn convert_mode(
    input: &str,
    mode: Base64ImageMode,
) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err(match mode {
            Base64ImageMode::Encode => "粘贴图片文件路径".into(),
            Base64ImageMode::Decode => "粘贴 data:image/…;base64,… 或纯 Base64".into(),
        });
    }
    match mode {
        Base64ImageMode::Encode => encode_path(t.lines().next().unwrap_or(t).trim()),
        Base64ImageMode::Decode => decode_payload(t),
    }
}

fn encode_path(path: &str) -> Result<Vec<(&'static str, String)>, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("无法访问 `{path}`：{e}"))?;
    if !meta.is_file() {
        return Err(format!("`{path}` 不是普通文件"));
    }
    if meta.len() > 6 * 1024 * 1024 {
        return Err("文件超过 6 MiB，拒绝读入".into());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("读取失败：{e}"))?;
    let mime = sniff_mime(&bytes).unwrap_or_else(|| mime_from_ext(path));
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:{mime};base64,{b64}");
    Ok(vec![
        ("MIME", mime.to_string()),
        ("字节数", bytes.len().to_string()),
        ("文件名", Path::new(path).file_name().and_then(|s| s.to_str()).unwrap_or(path).into()),
        ("Data URL", data_url),
        ("Base64", b64),
    ])
}

fn decode_payload(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let (mime_hint, b64) = split_data_url(input);
    let compact: String = b64.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.is_empty() {
        return Err("缺少 Base64 正文".into());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&compact)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(&compact))
        .map_err(|e| format!("Base64 解码失败：{e}"))?;
    if bytes.len() > 8 * 1024 * 1024 {
        return Err("解码后超过 8 MiB".into());
    }
    let mime = sniff_mime(&bytes).unwrap_or(mime_hint.unwrap_or("application/octet-stream"));
    let mut rows = vec![
        ("MIME", mime.to_string()),
        ("字节数", bytes.len().to_string()),
        ("魔数", hex_prefix(&bytes, 8)),
    ];
    if mime == "image/svg+xml" {
        if let Ok(s) = std::str::from_utf8(&bytes) {
            rows.push(("SVG", s.chars().take(4000).collect()));
        }
    } else {
        rows.push((
            "说明",
            "光栅预览未内嵌（无 GPU 图床）；可复制 Base64 到别处查看".into(),
        ));
        rows.push(("Base64", base64::engine::general_purpose::STANDARD.encode(&bytes)));
    }
    Ok(rows)
}

fn split_data_url(input: &str) -> (Option<&'static str>, &str) {
    let t = input.trim();
    if let Some(rest) = t.strip_prefix("data:")
        && let Some((meta, data)) = rest.split_once(',')
    {
        let mime = if meta.starts_with("image/png") {
            Some("image/png")
        } else if meta.starts_with("image/jpeg") || meta.starts_with("image/jpg") {
            Some("image/jpeg")
        } else if meta.starts_with("image/gif") {
            Some("image/gif")
        } else if meta.starts_with("image/webp") {
            Some("image/webp")
        } else if meta.starts_with("image/svg") {
            Some("image/svg+xml")
        } else {
            None
        };
        return (mime, data);
    }
    (None, t)
}

fn sniff_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']) {
        return Some("image/png");
    }
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("image/jpeg");
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return Some("image/gif");
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    let head = std::str::from_utf8(bytes.get(..64).unwrap_or(bytes)).unwrap_or("");
    if head.trim_start().starts_with("<svg") || head.contains("<svg") {
        return Some("image/svg+xml");
    }
    None
}

fn mime_from_ext(path: &str) -> &'static str {
    match Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn looks_b64(t: &str) -> bool {
    let c: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    c.len() >= 32
        && c.chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '/' | '=' | '-' | '_'))
}

fn hex_prefix(bytes: &[u8], n: usize) -> String {
    bytes
        .iter()
        .take(n)
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_png() {
        let mut png = vec![0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'];
        png.extend_from_slice(&[0; 8]);
        assert_eq!(sniff_mime(&png), Some("image/png"));
    }

    #[test]
    fn suggest_data_url_decode() {
        assert_eq!(
            suggest_mode("data:image/png;base64,aaaa"),
            Base64ImageMode::Decode
        );
    }
}
