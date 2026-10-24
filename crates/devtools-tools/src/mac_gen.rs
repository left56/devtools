//! Random MAC address generator (no OUI vendor lookup).

use uuid::Uuid;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::MAC_GEN,
        name: "MAC generator",
        category: Category::Generators,
        summary: "生成随机 MAC。可选单播/本地管理位；不做厂商 OUI 查询。",
        keywords: &["mac", "ethernet", "address", "网卡"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn generate(locally_administered: bool, unicast: bool) -> Vec<(&'static str, String)> {
    let uuid = Uuid::new_v4();
    let b = uuid.as_bytes();
    let mut mac = [b[0], b[1], b[2], b[3], b[4], b[5]];
    if unicast {
        mac[0] &= !0x01; // clear multicast bit
    } else {
        mac[0] |= 0x01;
    }
    if locally_administered {
        mac[0] |= 0x02;
    } else {
        mac[0] &= !0x02;
    }
    let colon = mac.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join(":");
    let dash = mac.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join("-");
    let bare = mac.iter().map(|x| format!("{x:02x}")).collect::<Vec<_>>().join("");
    let cisco = format!(
        "{:02x}{:02x}.{:02x}{:02x}.{:02x}{:02x}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );
    vec![
        ("冒号", colon),
        ("连字符", dash),
        ("无分隔", bare),
        ("Cisco", cisco),
        (
            "标志",
            format!(
                "{} · {}",
                if unicast { "单播" } else { "组播" },
                if locally_administered {
                    "本地管理"
                } else {
                    "全球唯一"
                }
            ),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicast_local() {
        let rows = generate(true, true);
        let first = u8::from_str_radix(&rows[0].1[0..2], 16).unwrap();
        assert_eq!(first & 0x01, 0);
        assert_eq!(first & 0x02, 0x02);
    }
}
