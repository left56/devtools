//! ULID generator (Crockford Base32, 128-bit). Offline; explicit generate.

use std::time::{SystemTime, UNIX_EPOCH};

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use uuid::Uuid;

const CROCKFORD: &[u8; 32] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::ULID,
        name: "ULID",
        category: Category::Generators,
        summary: "生成 ULID（时间有序、Crockford Base32）。点击生成，不自动产生。",
        keywords: &["ulid", "id", "生成", "sortable"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Generate a new ULID string (26 Crockford characters).
pub fn generate() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
        & 0xFFFF_FFFF_FFFF; // 48-bit
    let rand = Uuid::new_v4();
    let bytes = rand.as_bytes();
    let mut raw = [0u8; 16];
    raw[0] = ((ms >> 40) & 0xFF) as u8;
    raw[1] = ((ms >> 32) & 0xFF) as u8;
    raw[2] = ((ms >> 24) & 0xFF) as u8;
    raw[3] = ((ms >> 16) & 0xFF) as u8;
    raw[4] = ((ms >> 8) & 0xFF) as u8;
    raw[5] = (ms & 0xFF) as u8;
    raw[6..16].copy_from_slice(&bytes[0..10]);
    encode_crockford(&raw)
}

fn encode_crockford(bytes: &[u8; 16]) -> String {
    let mut buffer: u128 = 0;
    for b in bytes {
        buffer = (buffer << 8) | u128::from(*b);
    }
    // 128 bits → 26 × 5-bit symbols (top symbol uses 2 zero pad bits).
    let mut out = String::with_capacity(26);
    for i in (0..26).rev() {
        let shift = i * 5;
        let idx = ((buffer >> shift) & 0x1F) as usize;
        out.push(CROCKFORD[idx] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_length_and_alphabet() {
        let id = generate();
        assert_eq!(id.len(), 26);
        assert!(id.bytes().all(|b| CROCKFORD.contains(&b)));
        assert_ne!(id, generate());
    }

    #[test]
    fn time_ordered_roughly() {
        let a = generate();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = generate();
        assert!(a <= b || a[..10] <= b[..10]);
    }
}
