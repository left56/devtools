//! Light User-Agent heuristic parser.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::UA_PARSE,
        name: "User-Agent",
        category: Category::Web,
        summary: "轻量 User-Agent 解析（浏览器/系统启发式，非完整 UA 数据库）。",
        keywords: &["user-agent", "ua", "browser", "浏览器"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let ua = input.trim();
    if ua.is_empty() {
        return Err("输入为空".into());
    }
    let browser = detect_browser(ua);
    let os = detect_os(ua);
    let device = if ua.to_ascii_lowercase().contains("mobile")
        || ua.contains("Android")
        || ua.contains("iPhone")
    {
        "移动"
    } else if ua.contains("iPad") || ua.contains("Tablet") {
        "平板"
    } else {
        "桌面"
    };
    Ok(vec![
        ("浏览器", browser),
        ("系统", os),
        ("设备", device.into()),
        ("长度", ua.len().to_string()),
        ("含 Mozilla", if ua.contains("Mozilla") { "是" } else { "否" }.into()),
    ])
}

fn detect_browser(ua: &str) -> String {
    let checks = [
        ("Edg/", "Edge"),
        ("OPR/", "Opera"),
        ("Opera", "Opera"),
        ("Firefox/", "Firefox"),
        ("FxiOS/", "Firefox"),
        ("Chrome/", "Chrome"),
        ("CriOS/", "Chrome"),
        ("Safari/", "Safari"),
        ("MSIE ", "IE"),
        ("Trident/", "IE"),
    ];
    for (needle, name) in checks {
        if let Some(idx) = ua.find(needle) {
            let rest = &ua[idx + needle.len()..];
            let ver: String = rest
                .chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            return if ver.is_empty() {
                name.into()
            } else {
                format!("{name} {ver}")
            };
        }
    }
    "未知".into()
}

fn detect_os(ua: &str) -> String {
    if ua.contains("Windows NT 10") {
        "Windows 10/11".into()
    } else if ua.contains("Windows") {
        "Windows".into()
    } else if ua.contains("Android") {
        "Android".into()
    } else if ua.contains("iPhone") || ua.contains("iPad") || ua.contains("iOS") {
        "iOS".into()
    } else if ua.contains("Mac OS X") || ua.contains("Macintosh") {
        "macOS".into()
    } else if ua.contains("Linux") {
        "Linux".into()
    } else {
        "未知".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn chrome_windows() {
        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";
        let rows = convert_all(ua).unwrap();
        assert!(rows[0].1.starts_with("Chrome"));
        assert!(rows[1].1.contains("Windows"));
    }
}
