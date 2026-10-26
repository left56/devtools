//! File path ↔ Base64 (paste a path; no native file picker in GPUI 0.2).

use base64::Engine;
use std::path::Path;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base64FileMode {
    Encode,
    Decode,
}

impl Base64FileMode {
    pub const ALL: [Base64FileMode; 2] = [Base64FileMode::Encode, Base64FileMode::Decode];

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
        id: ids::BASE64_FILE,
        name: "Base64 file",
        category: Category::Encoding,
        summary: "文件路径 ↔ Base64。编码：拖入文件或粘贴路径；解码：粘贴 Base64，可选「路径\\nBase64」写出。",
        keywords: &["base64", "file", "binary", "文件", "路径", "拖拽"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn suggest_mode(input: &str) -> Base64FileMode {
    let t = input.trim();
    if t.is_empty() {
        return Base64FileMode::Encode;
    }
    // Path-like → encode; otherwise if looks like base64 → decode
    let first = t.lines().next().unwrap_or("").trim();
    if looks_like_path(first) && Path::new(first).exists() {
        return Base64FileMode::Encode;
    }
    let compact: String = t.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.len() >= 8 && looks_like_base64(&compact) {
        return Base64FileMode::Decode;
    }
    Base64FileMode::Encode
}

pub fn convert_mode(
    input: &str,
    mode: Base64FileMode,
) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err(match mode {
            Base64FileMode::Encode => "粘贴要编码的文件路径".into(),
            Base64FileMode::Decode => "粘贴 Base64；若要写回磁盘用「路径\\nBase64」".into(),
        });
    }
    match mode {
        Base64FileMode::Encode => encode_path(t.lines().next().unwrap_or(t).trim()),
        Base64FileMode::Decode => decode_payload(t),
    }
}

fn encode_path(path: &str) -> Result<Vec<(&'static str, String)>, String> {
    if path.is_empty() {
        return Err("路径为空".into());
    }
    let meta = std::fs::metadata(path).map_err(|e| format!("无法访问 `{path}`：{e}"))?;
    if !meta.is_file() {
        return Err(format!("`{path}` 不是普通文件"));
    }
    if meta.len() > 8 * 1024 * 1024 {
        return Err("文件超过 8 MiB，拒绝读入（防误贴大文件）".into());
    }
    let bytes = std::fs::read(path).map_err(|e| format!("读取失败：{e}"))?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let name = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path)
        .to_string();
    Ok(vec![
        ("文件名", name),
        ("路径", path.to_string()),
        ("字节数", bytes.len().to_string()),
        ("Base64", b64),
        (
            "说明",
            "标准 Base64；解码模式可写回文件".into(),
        ),
    ])
}

fn decode_payload(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let (out_path, b64_src) = split_path_and_b64(input);
    let compact: String = b64_src.chars().filter(|c| !c.is_whitespace()).collect();
    if compact.is_empty() {
        return Err("缺少 Base64 正文".into());
    }
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&compact)
        .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(&compact))
        .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&compact))
        .map_err(|e| format!("Base64 解码失败：{e}"))?;

    let mut rows = vec![
        ("字节数", bytes.len().to_string()),
        (
            "Hex 预览",
            bytes
                .iter()
                .take(64)
                .map(|b| format!("{b:02x}"))
                .collect::<Vec<_>>()
                .join(if bytes.len() > 32 { "" } else { " "}),
        ),
    ];
    match std::str::from_utf8(&bytes) {
        Ok(s) if s.chars().all(|c| !c.is_control() || matches!(c, '\n' | '\r' | '\t')) => {
            let preview: String = s.chars().take(200).collect();
            rows.push(("UTF-8 预览", preview));
        }
        _ => rows.push(("UTF-8 预览", "（非可打印 UTF-8）".into())),
    }

    if let Some(path) = out_path {
        if path.is_empty() {
            return Err("写出路径为空".into());
        }
        if let Some(parent) = Path::new(path).parent()
            && !parent.as_os_str().is_empty()
            && !parent.exists()
        {
            return Err(format!("目录不存在：{}", parent.display()));
        }
        std::fs::write(path, &bytes).map_err(|e| format!("写出失败：{e}"))?;
        rows.insert(0, ("已写入", path.to_string()));
    } else {
        rows.push((
            "提示",
            "若要写文件：第一行路径，其余为 Base64".into(),
        ));
    }
    Ok(rows)
}

fn split_path_and_b64(input: &str) -> (Option<&str>, &str) {
    let trimmed = input.trim();
    let Some(nl) = trimmed.find('\n') else {
        return (None, trimmed);
    };
    let first = trimmed[..nl].trim();
    let rest = trimmed[nl + 1..].trim();
    if !rest.is_empty() && looks_like_path(first) {
        (Some(first), rest)
    } else {
        (None, trimmed)
    }
}

fn looks_like_path(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    s.starts_with('/')
        || s.starts_with("./")
        || s.starts_with("../")
        || s.starts_with('~')
        || (s.len() >= 3 && s.as_bytes()[1] == b':' && s.as_bytes()[0].is_ascii_alphabetic())
        || s.contains('/')
        || s.contains('\\')
}

fn looks_like_base64(s: &str) -> bool {
    let ok = s
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'=' | b'-' | b'_'));
    ok && (s.contains('+')
        || s.contains('/')
        || s.contains('-')
        || s.contains('_')
        || s.ends_with('=')
        || s.len().is_multiple_of(4))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn encode_and_decode_round_trip() {
        let dir = std::env::temp_dir().join("devtools-b64-file-test");
        let _ = std::fs::create_dir_all(&dir);
        let path = dir.join("sample.bin");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"hello-file")
            .unwrap();
        let enc = convert_mode(path.to_str().unwrap(), Base64FileMode::Encode).unwrap();
        let b64 = enc.iter().find(|r| r.0 == "Base64").unwrap().1.clone();
        assert_eq!(b64, base64::engine::general_purpose::STANDARD.encode(b"hello-file"));

        let out = dir.join("out.bin");
        let payload = format!("{}\n{b64}", out.to_str().unwrap());
        let dec = convert_mode(&payload, Base64FileMode::Decode).unwrap();
        assert!(dec.iter().any(|r| r.0 == "已写入"));
        assert_eq!(std::fs::read(&out).unwrap(), b"hello-file");
    }

    #[test]
    fn decode_preview_without_path() {
        let b64 = base64::engine::general_purpose::STANDARD.encode(b"abc");
        let rows = convert_mode(&b64, Base64FileMode::Decode).unwrap();
        assert_eq!(rows.iter().find(|r| r.0 == "字节数").unwrap().1, "3");
    }
}
