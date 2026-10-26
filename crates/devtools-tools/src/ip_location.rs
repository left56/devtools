//! IP geolocation via Explicit HTTPS APIs (fallback chain, no bundled MMDB).
//!
//! Providers (in order): ip.sb → ipwho.is → ipapi.is → ifconfig.cc

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
        summary: "查询 IP 地理位置。依次尝试 ip.sb / ipwho.is / ipapi.is / ifconfig.cc；点击「运行」才联网。",
        keywords: &[
            "ip",
            "geo",
            "location",
            "geolocation",
            "ip.sb",
            "ipwho",
            "定位",
            "地理位置",
        ],
        network: NetworkUse::Explicit,
        availability: Availability::Ready,
    }
}

pub fn lookup(ip: &str) -> Result<Vec<(String, String)>, String> {
    let trimmed = ip.trim();
    // Empty → ask providers for the caller's public IP.
    let addr = if trimmed.is_empty() {
        None
    } else {
        Some(IpAddr::from_str(trimmed).map_err(|_| format!("不是合法 IP：{trimmed}"))?)
    };

    let mut errors = Vec::new();
    for provider in [
        Provider::IpSb,
        Provider::IpWhoIs,
        Provider::IpApiIs,
        Provider::IfconfigCc,
    ] {
        match provider.lookup(addr) {
            Ok(mut rows) => {
                rows.push(("来源".into(), provider.label().into()));
                return Ok(rows);
            }
            Err(e) => errors.push(format!("{}：{e}", provider.label())),
        }
    }
    Err(format!(
        "全部接口失败：\n{}",
        errors
            .into_iter()
            .map(|e| format!("· {e}"))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

#[derive(Clone, Copy)]
enum Provider {
    IpSb,
    IpWhoIs,
    IpApiIs,
    IfconfigCc,
}

impl Provider {
    fn label(self) -> &'static str {
        match self {
            Self::IpSb => "ip.sb",
            Self::IpWhoIs => "ipwho.is",
            Self::IpApiIs => "ipapi.is",
            Self::IfconfigCc => "ifconfig.cc",
        }
    }

    fn lookup(self, addr: Option<IpAddr>) -> Result<Vec<(String, String)>, String> {
        let url = self.url(addr);
        let body = http_get(&url)?;
        match self {
            Self::IpSb => parse_ip_sb(&body, addr),
            Self::IpWhoIs => parse_ipwho(&body, addr),
            Self::IpApiIs => parse_ipapi_is(&body, addr),
            Self::IfconfigCc => parse_ifconfig_cc(&body, addr),
        }
    }

    fn url(self, addr: Option<IpAddr>) -> String {
        match (self, addr) {
            (Self::IpSb, Some(ip)) => format!("https://api.ip.sb/geoip/{ip}"),
            (Self::IpSb, None) => "https://api.ip.sb/geoip".into(),
            (Self::IpWhoIs, Some(ip)) => format!("https://ipwho.is/{ip}"),
            (Self::IpWhoIs, None) => "https://ipwho.is/".into(),
            (Self::IpApiIs, Some(ip)) => format!("https://api.ipapi.is?q={ip}"),
            (Self::IpApiIs, None) => "https://api.ipapi.is".into(),
            (Self::IfconfigCc, Some(ip)) => format!("https://ifconfig.cc/json?ip={ip}"),
            (Self::IfconfigCc, None) => "https://ifconfig.cc/json".into(),
        }
    }
}

fn http_get(url: &str) -> Result<String, String> {
    ureq::get(url)
        .timeout(Duration::from_secs(10))
        .set("User-Agent", "devtools/0.1 (Explicit run)")
        .set("Accept", "application/json")
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())
}

fn parse_ip_sb(body: &str, fallback_ip: Option<IpAddr>) -> Result<Vec<(String, String)>, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if v.get("ip").is_none() && v.get("country").is_none() && v.get("country_code").is_none() {
        return Err(format!("无地理字段：{}", truncate(body, 160)));
    }
    let mut rows = Vec::new();
    push_ip(&mut rows, &v, fallback_ip);
    push(&mut rows, "国家", first(&v, &["country", "country_name"]));
    push(&mut rows, "国家代码", first(&v, &["country_code", "country_code_iso"]));
    push(&mut rows, "地区", first(&v, &["region", "region_name"]));
    push(&mut rows, "城市", v.get("city"));
    push(&mut rows, "邮编", first(&v, &["postal", "postal_code"]));
    push(&mut rows, "纬度", first(&v, &["latitude", "lat"]));
    push(&mut rows, "经度", first(&v, &["longitude", "lng", "lon"]));
    push(&mut rows, "时区", v.get("timezone"));
    push(&mut rows, "ASN", first(&v, &["asn", "organization_asn"]));
    push(&mut rows, "组织", first(&v, &["organization", "isp", "org"]));
    ensure_geo(&rows, body)?;
    Ok(rows)
}

fn parse_ipwho(body: &str, fallback_ip: Option<IpAddr>) -> Result<Vec<(String, String)>, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if v.get("success").and_then(|x| x.as_bool()) == Some(false) {
        let msg = v
            .get("message")
            .and_then(|x| x.as_str())
            .unwrap_or("查询失败");
        return Err(msg.into());
    }
    let mut rows = Vec::new();
    push_ip(&mut rows, &v, fallback_ip);
    push(&mut rows, "国家", first(&v, &["country"]));
    push(&mut rows, "国家代码", first(&v, &["country_code"]));
    push(&mut rows, "地区", first(&v, &["region", "regionName"]));
    push(&mut rows, "城市", v.get("city"));
    push(&mut rows, "邮编", v.get("postal"));
    push(&mut rows, "纬度", first(&v, &["latitude", "lat"]));
    push(&mut rows, "经度", first(&v, &["longitude", "lon"]));
    if let Some(tz) = v.get("timezone") {
        if let Some(id) = tz.get("id") {
            push(&mut rows, "时区", Some(id));
        } else {
            push(&mut rows, "时区", Some(tz));
        }
    }
    push(&mut rows, "ASN", first(&v, &["asn", "connection"]));
    if let Some(conn) = v.get("connection") {
        push(&mut rows, "组织", first(conn, &["org", "isp"]));
        push(&mut rows, "ASN", conn.get("asn"));
    } else {
        push(&mut rows, "组织", first(&v, &["org", "isp"]));
    }
    ensure_geo(&rows, body)?;
    Ok(rows)
}

fn parse_ipapi_is(body: &str, fallback_ip: Option<IpAddr>) -> Result<Vec<(String, String)>, String> {
    let v: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
    if let Some(err) = v.get("error").filter(|e| !e.is_null()) {
        return Err(err.to_string());
    }
    // ipapi.is nests location under "location" and company under "company".
    let loc = v.get("location").unwrap_or(&v);
    let mut rows = Vec::new();
    push_ip(&mut rows, &v, fallback_ip);
    push(&mut rows, "国家", first(loc, &["country", "country_name"]));
    push(
        &mut rows,
        "国家代码",
        first(loc, &["country_code", "country_code_iso"]),
    );
    push(&mut rows, "地区", first(loc, &["state", "region", "province"]));
    push(&mut rows, "城市", loc.get("city"));
    push(&mut rows, "邮编", loc.get("zip").or_else(|| loc.get("postal")));
    push(&mut rows, "纬度", first(loc, &["latitude", "lat"]));
    push(&mut rows, "经度", first(loc, &["longitude", "lng", "lon"]));
    push(&mut rows, "时区", loc.get("timezone").or_else(|| v.get("timezone")));
    if let Some(asn) = v.get("asn") {
        push(&mut rows, "ASN", asn.get("asn").or(Some(asn)));
        push(&mut rows, "组织", asn.get("org").or_else(|| asn.get("name")));
    }
    if let Some(co) = v.get("company") {
        push(&mut rows, "组织", co.get("name").or_else(|| co.get("org")));
    }
    ensure_geo(&rows, body)?;
    Ok(rows)
}

fn parse_ifconfig_cc(
    body: &str,
    fallback_ip: Option<IpAddr>,
) -> Result<Vec<(String, String)>, String> {
    // ifconfig.cc may return JSON or plain text IP; try JSON first.
    if let Ok(v) = serde_json::from_str::<Value>(body) {
        let mut rows = Vec::new();
        push_ip(&mut rows, &v, fallback_ip);
        push(&mut rows, "国家", first(&v, &["country", "country_name", "country_iso"]));
        push(&mut rows, "国家代码", first(&v, &["country_iso", "country_code", "cc"]));
        push(&mut rows, "地区", first(&v, &["region", "region_name"]));
        push(&mut rows, "城市", v.get("city"));
        push(&mut rows, "邮编", v.get("postal"));
        push(&mut rows, "纬度", first(&v, &["latitude", "lat"]));
        push(&mut rows, "经度", first(&v, &["longitude", "lon", "lng"]));
        push(&mut rows, "时区", v.get("timezone"));
        push(&mut rows, "ASN", first(&v, &["asn", "as_number"]));
        push(&mut rows, "组织", first(&v, &["org", "organization", "isp"]));
        ensure_geo(&rows, body)?;
        return Ok(rows);
    }
    let ip = body.trim();
    if IpAddr::from_str(ip).is_ok() {
        return Ok(vec![
            ("IP".into(), ip.into()),
            (
                "说明".into(),
                "ifconfig.cc 仅返回 IP，无地理字段；已尝试作为末位回退".into(),
            ),
        ]);
    }
    Err(format!("无法解析：{}", truncate(body, 160)))
}

fn push_ip(rows: &mut Vec<(String, String)>, v: &Value, fallback: Option<IpAddr>) {
    if let Some(ip) = first(v, &["ip", "query", "IP"]) {
        push(rows, "IP", Some(ip));
    } else if let Some(ip) = fallback {
        rows.push(("IP".into(), ip.to_string()));
    }
}

fn first<'a>(v: &'a Value, keys: &[&str]) -> Option<&'a Value> {
    keys.iter().find_map(|k| v.get(*k))
}

fn push(rows: &mut Vec<(String, String)>, label: &str, value: Option<&Value>) {
    let Some(v) = value else { return };
    let text = match v {
        Value::Null => return,
        Value::String(s) if s.is_empty() => return,
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Object(_) | Value::Array(_) => return,
    };
    // Avoid duplicate labels (keep first).
    if rows.iter().any(|(k, _)| k == label) {
        return;
    }
    rows.push((label.to_string(), text));
}

fn ensure_geo(rows: &[(String, String)], body: &str) -> Result<(), String> {
    let has_geo = rows.iter().any(|(k, _)| {
        matches!(
            k.as_str(),
            "国家" | "国家代码" | "城市" | "地区" | "纬度" | "经度"
        )
    });
    if has_geo {
        return Ok(());
    }
    if rows.iter().any(|(k, _)| k == "IP") {
        // Last-resort providers may only echo the IP.
        return Ok(());
    }
    Err(format!("响应缺少地理字段：{}", truncate(body, 160)))
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
    fn rejects_garbage_ip() {
        assert!(lookup("not-an-ip").is_err());
    }

    #[test]
    fn parse_ipwho_sample() {
        let body = r#"{"success":true,"ip":"8.8.8.8","country":"United States","country_code":"US","city":"Mountain View","latitude":37.38,"longitude":-122.07}"#;
        let rows = parse_ipwho(body, None).unwrap();
        assert!(rows.iter().any(|(k, v)| k == "城市" && v.contains("Mountain")));
    }

    #[test]
    fn parse_ip_sb_sample() {
        let body = r#"{"ip":"1.1.1.1","country":"Australia","country_code":"AU","city":"Sydney","latitude":-33.86,"longitude":151.20}"#;
        let rows = parse_ip_sb(body, None).unwrap();
        assert!(rows.iter().any(|(k, _)| k == "国家代码"));
    }

    #[test]
    fn meta_is_explicit() {
        assert_eq!(meta().network, NetworkUse::Explicit);
        assert_eq!(meta().id, ids::IP_LOCATION);
    }
}
