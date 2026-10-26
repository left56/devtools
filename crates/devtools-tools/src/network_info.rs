//! Local network configuration (+ optional public IP via Explicit HTTPS).

use std::time::Duration;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::NETWORK_INFO,
        name: "Network Information",
        category: Category::Web,
        summary: "本机网卡 / 地址摘要；「运行」时可选拉取公网 IP（会联网）。Linux 优先读 /proc。",
        keywords: &["network", "interface", "ip", "gateway", "dns", "网卡", "公网"],
        network: NetworkUse::Explicit,
        availability: Availability::Ready,
    }
}

/// Gather local details always; fetch public IP when `fetch_public` is true.
pub fn collect(fetch_public: bool) -> Result<Vec<(String, String)>, String> {
    let mut rows = Vec::new();
    rows.push(("主机名".into(), hostname()));
    collect_local(&mut rows);
    if fetch_public {
        match public_ip() {
            Ok(ip) => rows.insert(0, ("公网 IP".into(), ip)),
            Err(e) => rows.push(("公网 IP".into(), format!("（失败：{e}）"))),
        }
    } else {
        rows.push(("公网 IP".into(), "（未请求；本工具点击运行会尝试获取）".into()));
    }
    Ok(rows)
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .or_else(|_| std::env::var("COMPUTERNAME"))
        .unwrap_or_else(|_| {
            std::fs::read_to_string("/etc/hostname")
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| "（未知）".into())
        })
}

fn public_ip() -> Result<String, String> {
    let body = ureq::get("https://checkip.amazonaws.com/")
        .timeout(Duration::from_secs(8))
        .set("User-Agent", "devtools/0.1 (Explicit run)")
        .call()
        .map_err(|e| e.to_string())?
        .into_string()
        .map_err(|e| e.to_string())?;
    let ip = body.trim().to_string();
    if ip.is_empty() {
        return Err("空响应".into());
    }
    Ok(ip)
}

#[cfg(target_os = "linux")]
fn collect_local(rows: &mut Vec<(String, String)>) {
    // Interfaces via `ip -br addr` when available; else /proc/net/fib_trie skim.
    if let Ok(output) = std::process::Command::new("ip")
        .args(["-br", "addr"])
        .output()
        && output.status.success()
    {
        let text = String::from_utf8_lossy(&output.stdout);
        let mut n = 0;
        for line in text.lines().take(12) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            n += 1;
            rows.push((format!("接口 {n}"), line.to_string()));
        }
    }
    if let Ok(text) = std::fs::read_to_string("/etc/resolv.conf") {
        for line in text.lines() {
            let line = line.trim();
            if let Some(dns) = line.strip_prefix("nameserver") {
                rows.push(("DNS".into(), dns.trim().to_string()));
            }
        }
    }
    if let Ok(text) = std::fs::read_to_string("/proc/net/route") {
        for line in text.lines().skip(1) {
            let cols: Vec<_> = line.split_whitespace().collect();
            if cols.len() >= 3
                && cols[1] == "00000000"
                && let Ok(gw) = u32::from_str_radix(cols[2], 16)
            {
                let ip = format!(
                    "{}.{}.{}.{}",
                    gw & 0xff,
                    (gw >> 8) & 0xff,
                    (gw >> 16) & 0xff,
                    (gw >> 24) & 0xff
                );
                rows.push(("默认网关".into(), ip));
                break;
            }
        }
    }
    if rows.iter().all(|(k, _)| k != "接口 1") {
        rows.push((
            "提示".into(),
            "未读到网卡列表（需要 iproute2 的 ip 命令）".into(),
        ));
    }
}

#[cfg(not(target_os = "linux"))]
fn collect_local(rows: &mut Vec<(String, String)>) {
    rows.push((
        "本机网卡".into(),
        "当前构建目标非 Linux；仅提供主机名与公网 IP。完整网卡/网关/DNS 解析在 Linux 优先实现。".into(),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_local_only_works() {
        let rows = collect(false).unwrap();
        assert!(rows.iter().any(|(k, _)| k == "主机名"));
    }

    #[test]
    fn meta_is_explicit() {
        assert_eq!(meta().network, NetworkUse::Explicit);
    }
}
