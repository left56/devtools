//! IPv6 ULA generator (fd00::/8).

use uuid::Uuid;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::IPV6_ULA,
        name: "IPv6 ULA",
        category: Category::Generators,
        summary: "生成 IPv6 唯一本地地址（ULA，fd00::/8）。点击生成。",
        keywords: &["ipv6", "ula", "fd00", "本地地址"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn generate() -> Vec<(&'static str, String)> {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let mut raw = [0u8; 16];
    raw[0] = 0xfd;
    raw[1..6].copy_from_slice(&bytes[0..5]);
    raw[6] = bytes[5];
    raw[7] = bytes[6];
    raw[8..16].copy_from_slice(&bytes[8..16]);
    let hextets: Vec<String> = raw
        .chunks(2)
        .map(|c| format!("{:02x}{:02x}", c[0], c[1]))
        .collect();
    let full = hextets.join(":");
    let prefix = format!(
        "fd{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}::/48",
        raw[1], raw[2], raw[3], raw[4], raw[5], 0
    );
    vec![
        ("ULA", full),
        ("前缀 /48", prefix),
        ("说明", "RFC 4193 唯一本地地址，仅站点内路由".into()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_with_fd() {
        let rows = generate();
        assert!(rows[0].1.starts_with("fd"));
        assert_eq!(rows[0].1.matches(':').count(), 7);
    }
}
