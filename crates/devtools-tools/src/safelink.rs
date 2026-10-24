//! Outlook SafeLinks / similar URL decoder (extract real URL).

use url::Url;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::SAFELINK,
        name: "Safelink decoder",
        category: Category::Web,
        summary: "从 Outlook SafeLinks 等包装链接中提取真实 URL。",
        keywords: &["safelink", "outlook", "url", "unwrap", "解码"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    let parsed = Url::parse(t).map_err(|e| format!("无法解析 URL：{e}"))?;
    let mut rows = vec![
        ("主机", parsed.host_str().unwrap_or("").to_string()),
        ("路径", parsed.path().to_string()),
    ];
    // Common SafeLinks: .../safelinks.protection.outlook.com/?url=
    if let Some(inner) = parsed
        .query_pairs()
        .find(|(k, _)| k.eq_ignore_ascii_case("url") || k.eq_ignore_ascii_case("u"))
        .map(|(_, v)| v.to_string())
    {
        let decoded = percent_encoding::percent_decode_str(&inner)
            .decode_utf8_lossy()
            .to_string();
        rows.insert(0, ("真实 URL", decoded));
        rows.push(("已解包", "是".into()));
    } else {
        rows.insert(0, ("真实 URL", t.to_string()));
        rows.push(("已解包", "否（无 url/u 参数）".into()));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn extract_url_param() {
        let rows = convert_all(
            "https://nam.safelinks.protection.outlook.com/?url=https%3A%2F%2Fexample.com%2Fa&data=1",
        )
        .unwrap();
        assert_eq!(
            rows.iter().find(|r| r.0 == "真实 URL").unwrap().1,
            "https://example.com/a"
        );
    }
}
