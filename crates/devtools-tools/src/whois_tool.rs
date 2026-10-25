//! Domain / IP WHOIS over TCP :43 (Explicit).

use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::WHOIS,
        name: "Whois",
        category: Category::Web,
        summary: "查询域名 / IP 的 WHOIS。点击「运行」才会连接 whois 服务器（TCP 43）。",
        keywords: &["whois", "domain", "注册", "域名"],
        network: NetworkUse::Explicit,
        availability: Availability::Ready,
    }
}

pub fn lookup(query: &str) -> Result<Vec<(String, String)>, String> {
    let q = query.trim().trim_end_matches('.').to_ascii_lowercase();
    if q.is_empty() {
        return Err("输入域名或 IP，例如 example.com".into());
    }
    if q.contains(char::is_whitespace) {
        return Err("查询串不能含空白".into());
    }
    let first = whois_query("whois.iana.org", &q)?;
    let server = extract_refer(&first).unwrap_or_else(|| default_server_for(&q));
    let body = if server.eq_ignore_ascii_case("whois.iana.org") {
        first.clone()
    } else {
        whois_query(&server, &q).unwrap_or(first.clone())
    };
    let mut rows = vec![
        ("查询".into(), q),
        ("服务器".into(), server),
    ];
    for (label, key) in [
        ("注册商", "Registrar:"),
        ("创建时间", "Creation Date:"),
        ("到期时间", "Registry Expiry Date:"),
        ("状态", "Domain Status:"),
        ("名称服务器", "Name Server:"),
    ] {
        if let Some(v) = find_field(&body, key) {
            rows.push((label.into(), v));
        }
    }
    rows.push(("原始响应".into(), body));
    Ok(rows)
}

fn default_server_for(q: &str) -> String {
    if q.parse::<std::net::IpAddr>().is_ok() {
        return "whois.arin.net".into();
    }
    let tld = q.rsplit('.').next().unwrap_or("com");
    match tld {
        "com" | "net" => "whois.verisign-grs.com".into(),
        "org" => "whois.pir.org".into(),
        "io" => "whois.nic.io".into(),
        "cn" => "whois.cnnic.cn".into(),
        _ => "whois.iana.org".into(),
    }
}

fn whois_query(server: &str, query: &str) -> Result<String, String> {
    let addr = format!("{server}:43")
        .to_socket_addrs()
        .map_err(|e| format!("解析 {server} 失败：{e}"))?
        .next()
        .ok_or_else(|| format!("无法解析 {server}"))?;
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_secs(8))
        .map_err(|e| format!("连接 {server}:43 失败：{e}"))?;
    let _ = stream.set_read_timeout(Some(Duration::from_secs(10)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(8)));
    stream
        .write_all(format!("{query}\r\n").as_bytes())
        .map_err(|e| format!("发送查询失败：{e}"))?;
    let mut buf = Vec::new();
    stream
        .take(256 * 1024)
        .read_to_end(&mut buf)
        .map_err(|e| format!("读取响应失败：{e}"))?;
    let text = String::from_utf8_lossy(&buf).into_owned();
    if text.trim().is_empty() {
        return Err("WHOIS 响应为空".into());
    }
    Ok(text)
}

fn extract_refer(body: &str) -> Option<String> {
    for line in body.lines() {
        let lower = line.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("refer:") {
            let s = rest.trim();
            if !s.is_empty() {
                return Some(s.to_string());
            }
        }
        if let Some(rest) = lower.strip_prefix("whois:") {
            let s = rest.trim();
            if !s.is_empty() && s.contains('.') {
                return Some(s.to_string());
            }
        }
    }
    None
}

fn find_field(body: &str, key: &str) -> Option<String> {
    let key_l = key.to_ascii_lowercase();
    for line in body.lines() {
        if line.to_ascii_lowercase().starts_with(&key_l) {
            let v = line
                .split_once(':')
                .map(|(_, v)| v.trim())
                .unwrap_or("")
                .to_string();
            if !v.is_empty() {
                return Some(v);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty() {
        assert!(lookup("").is_err());
        assert!(lookup("a b").is_err());
    }

    #[test]
    fn meta_is_explicit() {
        assert_eq!(meta().network, NetworkUse::Explicit);
    }
}
