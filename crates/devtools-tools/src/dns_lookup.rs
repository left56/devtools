//! DNS lookup. Network is contacted only on explicit run.

use std::fmt;
use std::net::IpAddr;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordKind {
    /// Resolve A and AAAA via the system resolver.
    Address,
}

impl RecordKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Address => "A/AAAA",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsError {
    pub message: String,
}

impl fmt::Display for DnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for DnsError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsResult {
    pub name: String,
    pub kind: RecordKind,
    pub addresses: Vec<IpAddr>,
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::DNS_LOOKUP,
        name: "DNS Lookup",
        category: Category::Web,
        summary: "显式查询主机名的 A/AAAA。打开工具不会联网。",
        keywords: &["dns", "lookup", "resolve", "a", "aaaa", "域名", "联网"],
        network: NetworkUse::Explicit,
        availability: Availability::Ready,
    }
}

/// Validate the hostname / name without contacting the network.
pub fn prepare_name(input: &str) -> Result<String, DnsError> {
    let name = input.trim().trim_end_matches('.').to_owned();
    if name.is_empty() {
        return Err(DnsError {
            message: "主机名为空".into(),
        });
    }
    if name.contains(char::is_whitespace) {
        return Err(DnsError {
            message: "主机名不能包含空白".into(),
        });
    }
    if name.contains('/') || name.contains(':') {
        // Avoid accidental URL paste being treated as a name without parsing.
        // IPv6 literals are allowed separately via lookup of raw IPs below.
        if name.chars().any(|c| c.is_ascii_alphabetic()) {
            return Err(DnsError {
                message: "请输入主机名，不要粘贴带路径的 URL".into(),
            });
        }
    }
    if name.len() > 253 {
        return Err(DnsError {
            message: "主机名过长".into(),
        });
    }
    Ok(name)
}

/// Live lookup via the system resolver. Explicit run only.
pub fn lookup(name: &str, kind: RecordKind) -> Result<DnsResult, DnsError> {
    let name = prepare_name(name)?;
    match kind {
        RecordKind::Address => {
            let ips = dns_lookup::lookup_host(&name).map_err(|e| DnsError {
                message: format!("解析失败：{e}"),
            })?;
            if ips.is_empty() {
                return Err(DnsError {
                    message: "没有返回任何地址".into(),
                });
            }
            Ok(DnsResult {
                name,
                kind,
                addresses: ips,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meta_is_explicit() {
        let m = meta();
        assert_eq!(m.id, ids::DNS_LOOKUP);
        assert_eq!(m.network, NetworkUse::Explicit);
    }

    #[test]
    fn prepare_rejects_empty_and_urlish() {
        assert!(prepare_name("").is_err());
        assert!(prepare_name("https://example.com/path").is_err());
        assert!(prepare_name("example.com").is_ok());
        assert!(prepare_name("localhost").is_ok());
    }

    #[test]
    fn localhost_lookup_smoke() {
        // Explicit network path — localhost should resolve on any sane host.
        let result = lookup("localhost", RecordKind::Address).unwrap();
        assert!(!result.addresses.is_empty());
        assert!(
            result
                .addresses
                .iter()
                .any(|ip| ip.is_loopback() || matches!(ip, IpAddr::V4(v) if v.octets()[0] == 127))
        );
    }
}
