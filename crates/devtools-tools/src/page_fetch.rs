//! Fetch / crawl a single page (Explicit). Replaces the IndexNow-shaped need.

use std::io::Read;
use std::time::Duration;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::PAGE_FETCH,
        name: "抓取页面",
        category: Category::Web,
        summary: "抓取网页：状态码、响应头、标题与正文预览。点击「运行」才联网（原 IndexNow 意图）。",
        keywords: &[
            "fetch",
            "crawl",
            "scrape",
            "indexnow",
            "page",
            "html",
            "抓取",
            "爬取",
            "网页",
        ],
        network: NetworkUse::Explicit,
        availability: Availability::Ready,
    }
}

pub fn fetch(url_input: &str) -> Result<Vec<(String, String)>, String> {
    let raw = url_input.trim();
    if raw.is_empty() {
        return Err("输入要抓取的 URL，例如 https://example.com".into());
    }
    let url = if raw.starts_with("http://") || raw.starts_with("https://") {
        raw.to_string()
    } else {
        format!("https://{raw}")
    };
    let parsed = url::Url::parse(&url).map_err(|e| format!("URL 无效：{e}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err("仅支持 http / https".into());
    }

    let response = ureq::get(parsed.as_str())
        .timeout(Duration::from_secs(20))
        .set(
            "User-Agent",
            "devtools/0.1 (Page Fetch; Explicit run; +https://github.com/left56/devtools)",
        )
        .set("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .call()
        .map_err(|e| format!("请求失败：{e}"))?;

    let status = response.status();
    let status_text = response.status_text().to_string();
    let final_url = response.get_url().to_string();
    let mut header_rows = Vec::new();
    for name in [
        "content-type",
        "content-length",
        "server",
        "date",
        "location",
        "x-powered-by",
    ] {
        if let Some(v) = response.header(name) {
            header_rows.push((format!("头 · {name}"), v.to_string()));
        }
    }
    let content_type = response
        .header("content-type")
        .unwrap_or("")
        .to_ascii_lowercase();

    let mut body = Vec::new();
    response
        .into_reader()
        .take(512 * 1024)
        .read_to_end(&mut body)
        .map_err(|e| format!("读取正文失败：{e}"))?;

    let mut rows = vec![
        ("URL".into(), final_url),
        ("状态".into(), format!("{status} {status_text}")),
        ("字节数".into(), body.len().to_string()),
    ];
    rows.extend(header_rows);

    if content_type.contains("html") || looks_like_html(&body) {
        let text = String::from_utf8_lossy(&body);
        if let Some(title) = extract_title(&text) {
            rows.push(("标题".into(), title));
        }
        rows.push(("正文预览".into(), preview_text(&text, 4000)));
    } else if content_type.contains("json") || body.first() == Some(&b'{') || body.first() == Some(&b'[')
    {
        let text = String::from_utf8_lossy(&body);
        rows.push(("JSON 预览".into(), preview_text(&text, 4000)));
    } else if let Ok(text) = std::str::from_utf8(&body) {
        rows.push(("正文预览".into(), preview_text(text, 4000)));
    } else {
        rows.push((
            "说明".into(),
            "二进制响应；已截断至 512 KiB，未做解码预览".into(),
        ));
    }
    Ok(rows)
}


fn looks_like_html(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(256)]).to_ascii_lowercase();
    head.contains("<html") || head.contains("<!doctype html")
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let after = start + "<title".len();
    let after = html[after..]
        .find('>')
        .map(|i| after + i + 1)?;
    let end_rel = lower[after..].find("</title>")?;
    let title = html[after..after + end_rel].trim();
    if title.is_empty() {
        None
    } else {
        Some(collapse_ws(title))
    }
}

fn collapse_ws(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn preview_text(s: &str, max: usize) -> String {
    let t: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        format!("{t}\n…")
    } else {
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        assert!(fetch("").is_err());
    }

    #[test]
    fn extract_title_basic() {
        assert_eq!(
            extract_title("<html><head><title> Hello  World </title></head>"),
            Some("Hello World".into())
        );
    }

    #[test]
    fn meta_is_explicit() {
        assert_eq!(meta().network, NetworkUse::Explicit);
        assert_eq!(meta().id, ids::PAGE_FETCH);
    }
}
