//! IP geolocation via Explicit HTTPS lookup (no bundled MMDB).

use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use serde_json::Value;

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::IP_LOCATION,
        name: "IP Location",
        category: Category::Web,
        summary: "查询 IP 地理位置（国家/城市/坐标）。点击「运行」才会联网；未内置 MMDB。",
        keywords: &["ip", "geo", "location", "geolocation", "定位", "地理位置"],
        network: NetworkUse::Explicit,
        availability: Availability::Ready,
    }
}

pub fn lookup(ip: &str) -> Result<Vec<(String, String)>, String> {
    let trimmed = ip.trim();
    if trimmed.is_empty() {
        return Err("输入 IPv4 / IPv6 地址".into());
    }
    let addr = IpAddr::from_str(trimmed).map_err(|_| format!("不是合法 IP：{trimmed}"))?;
    // Free HTTPS JSON API (rate-limited). No API key.
    let url = format!("https://ipapi.co/{addr}/json/");
    let body = ureq::get(&url)
        .timeout(Duration::from_secs(12))
        .set("User-Agent", "devtools/0.1 (offline-toolbox; Explicit run)")
        .set("Accept", "application/json")
        .call()
        .map_err(|e| format!("联网失败：{e}"))?
        .into_string()
        .map_err(|e| format!("读取响应失败：{e}"))?;
    let v: Value = serde_json::from_str(&body).map_err(|e| format!("JSON 解析失败：{e}"))?;
    if let Some(reason) = v.get("reason").and_then(|x| x.as_str()) {
        return Err(format!("查询失败：{reason}"));
    }
    if v.get("error").and_then(|x| x.as_bool()) == Some(true) {
        let msg = v
            .get("message")
            .and_then(|x| x.as_str())
            .unwrap_or("未知错误");
        return Err(format!("查询失败：{msg}"));
    }
    let mut rows = Vec::new();
    push(&mut rows, "IP", v.get("ip").or(Some(&Value::String(addr.to_string()))));
    push(&mut rows, "国家", v.get("country_name").or_else(|| v.get("country")));
    push(&mut rows, "国家代码", v.get("country_code").or_else(|| v.get("country")));
    push(&mut rows, "地区", v.get("region").or_else(|| v.get("region_code")));
    push(&mut rows, "城市", v.get("city"));
    push(&mut rows, "邮编", v.get("postal"));
    push(&mut rows, "纬度", v.get("latitude"));
    push(&mut rows, "经度", v.get("longitude"));
    push(&mut rows, "时区", v.get("timezone"));
    push(&mut rows, "ASN", v.get("asn"));
    push(&mut rows, "组织", v.get("org"));
    push(&mut rows, "来源", Some(&Value::String("ipapi.co（显式联网）".into())));
    if rows.len() <= 2 {
        return Err(format!("响应缺少地理字段：{}", truncate(&body, 200)));
    }
    Ok(rows)
}

fn push(rows: &mut Vec<(String, String)>, label: &str, value: Option<&Value>) {
    let Some(v) = value else { return };
    let text = match v {
        Value::Null => return,
        Value::String(s) if s.is_empty() => return,
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    };
    rows.push((label.to_string(), text));
}

fn truncate(s: &str, n: usize) -> String {
    let t: String = s.chars().take(n).collect();
    if s.chars().count() > n {
        format!("{t}…")
    } else {
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_and_garbage() {
        assert!(lookup("").is_err());
        assert!(lookup("not-an-ip").is_err());
    }

    #[test]
    fn meta_is_explicit() {
        assert_eq!(meta().network, NetworkUse::Explicit);
        assert_eq!(meta().id, ids::IP_LOCATION);
    }
}
