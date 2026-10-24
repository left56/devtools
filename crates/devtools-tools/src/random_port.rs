//! Random TCP port generator (ephemeral range).

use uuid::Uuid;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::RANDOM_PORT,
        name: "Random port",
        category: Category::Generators,
        summary: "生成随机端口（默认 1024–65535）。点击生成。",
        keywords: &["port", "tcp", "random", "端口"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn generate() -> Vec<(&'static str, String)> {
    let uuid = Uuid::new_v4();
    let bytes = uuid.as_bytes();
    let n = u16::from_le_bytes([bytes[0], bytes[1]]);
    let port = 1024 + (n as u32 % (65535 - 1024 + 1)) as u16;
    let priv_port = 49152
        + (u16::from_le_bytes([bytes[2], bytes[3]]) as u32 % (65535 - 49152 + 1)) as u16;
    vec![
        ("端口 (1024–65535)", port.to_string()),
        ("动态端口 (49152–65535)", priv_port.to_string()),
        ("URL 示例", format!("http://127.0.0.1:{port}")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn in_range() {
        let rows = generate();
        let p: u16 = rows[0].1.parse().unwrap();
        assert!((1024..=65535).contains(&p));
    }
}
