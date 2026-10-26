//! IPv4 subnet calculator, address converter, range expander.

use std::net::Ipv4Addr;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn subnet_meta() -> ToolMeta {
    ToolMeta {
        id: ids::IPV4_SUBNET,
        name: "IPv4 subnet",
        category: Category::Web,
        summary: "IPv4 子网计算：CIDR → 掩码、网络、广播、可用主机。",
        keywords: &["ipv4", "subnet", "cidr", "子网", "掩码"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn addr_meta() -> ToolMeta {
    ToolMeta {
        id: ids::IPV4_ADDR,
        name: "IPv4 address",
        category: Category::Web,
        summary: "IPv4 地址互转：点分、十进制、二进制、十六进制。",
        keywords: &["ipv4", "address", "ip", "地址"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn range_meta() -> ToolMeta {
    ToolMeta {
        id: ids::IPV4_RANGE,
        name: "IPv4 range",
        category: Category::Web,
        summary: "IPv4 范围展开：起止地址 → CIDR 建议与数量。",
        keywords: &["ipv4", "range", "cidr", "范围"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

fn parse_ip(s: &str) -> Result<Ipv4Addr, String> {
    s.trim().parse().map_err(|_| format!("无效 IPv4：{s}"))
}

fn parse_cidr(input: &str) -> Result<(Ipv4Addr, u8), String> {
    let t = input.trim();
    let (ip_s, prefix_s) = t
        .split_once('/')
        .ok_or_else(|| "格式：x.x.x.x/前缀，例如 192.168.1.10/24".to_string())?;
    let ip = parse_ip(ip_s)?;
    let prefix: u8 = prefix_s
        .parse()
        .map_err(|_| "前缀须为 0–32".to_string())?;
    if prefix > 32 {
        return Err("前缀须为 0–32".into());
    }
    Ok((ip, prefix))
}

fn mask_from_prefix(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}

pub fn subnet_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let (ip, prefix) = parse_cidr(input)?;
    let ip_u = u32::from(ip);
    let mask = mask_from_prefix(prefix);
    let network = ip_u & mask;
    let broadcast = network | !mask;
    let hosts = if prefix >= 31 {
        0u64
    } else {
        (1u64 << (32 - prefix)) - 2
    };
    let first = if prefix >= 31 {
        network
    } else {
        network + 1
    };
    let last = if prefix >= 31 {
        broadcast
    } else {
        broadcast.saturating_sub(1)
    };
    Ok(vec![
        ("CIDR", format!("{}/{prefix}", Ipv4Addr::from(network))),
        ("子网掩码", Ipv4Addr::from(mask).to_string()),
        ("反掩码", Ipv4Addr::from(!mask).to_string()),
        ("网络地址", Ipv4Addr::from(network).to_string()),
        ("广播地址", Ipv4Addr::from(broadcast).to_string()),
        ("首个可用", Ipv4Addr::from(first).to_string()),
        ("末个可用", Ipv4Addr::from(last).to_string()),
        ("可用主机数", hosts.to_string()),
        ("输入地址", ip.to_string()),
    ])
}

pub fn addr_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    let ip = if let Ok(ip) = parse_ip(t) {
        ip
    } else if let Ok(n) = t.parse::<u32>() {
        Ipv4Addr::from(n)
    } else if let Ok(n) = u32::from_str_radix(t.trim_start_matches("0x"), 16) {
        Ipv4Addr::from(n)
    } else {
        return Err("支持点分、十进制或 0x 十六进制".into());
    };
    let n = u32::from(ip);
    let bin = format!("{n:032b}");
    let bin_dotted = format!(
        "{}.{}.{}.{}",
        &bin[0..8],
        &bin[8..16],
        &bin[16..24],
        &bin[24..32]
    );
    Ok(vec![
        ("点分十进制", ip.to_string()),
        ("整数", n.to_string()),
        ("十六进制", format!("0x{n:08x}")),
        ("二进制", bin_dotted),
    ])
}

pub fn range_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    let (a, b) = t
        .split_once(['-', ' ', ','])
        .ok_or_else(|| "格式：起始-结束，例如 192.168.1.0-192.168.1.255".to_string())?;
    let start = parse_ip(a)?;
    let end = parse_ip(b)?;
    let s = u32::from(start);
    let e = u32::from(end);
    if e < s {
        return Err("结束地址小于起始地址".into());
    }
    let count = u64::from(e - s) + 1;
    let exact = exact_cidr(s, e);
    let mut rows = vec![
        ("起始", start.to_string()),
        ("结束", end.to_string()),
        ("地址数", count.to_string()),
    ];
    if let Some((net, p)) = exact {
        rows.push(("精确 CIDR", format!("{}/{p}", Ipv4Addr::from(net))));
    } else {
        rows.push(("精确 CIDR", "非单一 CIDR（范围未对齐）".into()));
    }
    Ok(rows)
}

fn exact_cidr(start: u32, end: u32) -> Option<(u32, u8)> {
    let size = end.wrapping_sub(start).wrapping_add(1);
    if size == 0 || (size & (size - 1)) != 0 {
        return None;
    }
    let prefix = 32 - size.trailing_zeros() as u8;
    let mask = mask_from_prefix(prefix);
    if start & mask == start && (start | !mask) == end {
        Some((start, prefix))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn subnet_24() {
        let rows = subnet_all("192.168.1.10/24").unwrap();
        assert_eq!(
            rows.iter().find(|r| r.0 == "网络地址").unwrap().1,
            "192.168.1.0"
        );
        assert_eq!(
            rows.iter().find(|r| r.0 == "广播地址").unwrap().1,
            "192.168.1.255"
        );
        assert_eq!(
            rows.iter().find(|r| r.0 == "可用主机数").unwrap().1,
            "254"
        );
    }
    #[test]
    fn addr_decimal() {
        let rows = addr_all("3232235777").unwrap();
        assert_eq!(rows[0].1, "192.168.1.1");
    }
    #[test]
    fn range_exact() {
        let rows = range_all("192.168.1.0-192.168.1.255").unwrap();
        assert_eq!(
            rows.iter().find(|r| r.0 == "精确 CIDR").unwrap().1,
            "192.168.1.0/24"
        );
    }
}
