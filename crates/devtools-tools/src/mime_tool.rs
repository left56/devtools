//! Common MIME type lookup by extension or type.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::MIME,
        name: "MIME types",
        category: Category::Web,
        summary: "常见扩展名 ↔ MIME 类型查询（内置表，不联网）。",
        keywords: &["mime", "content-type", "extension", "扩展名"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const TABLE: &[(&str, &str)] = &[
    ("html", "text/html"),
    ("htm", "text/html"),
    ("css", "text/css"),
    ("js", "text/javascript"),
    ("mjs", "text/javascript"),
    ("json", "application/json"),
    ("xml", "application/xml"),
    ("txt", "text/plain"),
    ("md", "text/markdown"),
    ("csv", "text/csv"),
    ("png", "image/png"),
    ("jpg", "image/jpeg"),
    ("jpeg", "image/jpeg"),
    ("gif", "image/gif"),
    ("webp", "image/webp"),
    ("svg", "image/svg+xml"),
    ("ico", "image/x-icon"),
    ("mp3", "audio/mpeg"),
    ("wav", "audio/wav"),
    ("mp4", "video/mp4"),
    ("webm", "video/webm"),
    ("pdf", "application/pdf"),
    ("zip", "application/zip"),
    ("gz", "application/gzip"),
    ("tar", "application/x-tar"),
    ("wasm", "application/wasm"),
    ("woff", "font/woff"),
    ("woff2", "font/woff2"),
    ("ttf", "font/ttf"),
    ("otf", "font/otf"),
    ("bin", "application/octet-stream"),
];

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim().trim_start_matches('.').to_ascii_lowercase();
    if t.is_empty() {
        return Err("输入扩展名（html）或 MIME（text/html）".into());
    }
    if t.contains('/') {
        let mut matches: Vec<&str> = TABLE
            .iter()
            .filter(|(_, mime)| mime.eq_ignore_ascii_case(&t))
            .map(|(ext, _)| *ext)
            .collect();
        if matches.is_empty() {
            return Err(format!("未找到 MIME `{t}`"));
        }
        matches.sort();
        Ok(vec![
            ("MIME", t),
            ("扩展名", matches.join(", ")),
        ])
    } else {
        let mime = TABLE
            .iter()
            .find(|(ext, _)| *ext == t)
            .map(|(_, m)| *m)
            .ok_or_else(|| format!("未知扩展名 `.{t}`"))?;
        Ok(vec![
            ("扩展名", format!(".{t}")),
            ("MIME", mime.to_string()),
            ("Content-Type", mime.to_string()),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ext_and_mime() {
        assert_eq!(convert_all("png").unwrap()[1].1, "image/png");
        assert!(convert_all("image/png").unwrap()[1].1.contains("png"));
    }
}
