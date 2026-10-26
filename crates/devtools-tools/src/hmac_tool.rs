//! HMAC digests. Input: first line = key, rest = message.

use hmac::{Hmac, Mac};
use md5::Md5;
use sha1::Sha1;
use sha2::{Sha256, Sha512};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

type HmacMd5 = Hmac<Md5>;
type HmacSha1 = Hmac<Sha1>;
type HmacSha256 = Hmac<Sha256>;
type HmacSha512 = Hmac<Sha512>;

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::HMAC,
        name: "HMAC",
        category: Category::Security,
        summary: "HMAC-MD5/SHA-1/SHA-256/SHA-512。第一行密钥，其余为消息。",
        keywords: &["hmac", "mac", "签名", "keyed", "hash"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

fn split_key_msg(input: &str) -> Result<(&str, &str), String> {
    let trimmed = input.trim_end();
    if trimmed.is_empty() {
        return Err("输入为空：第一行密钥，其余为消息".into());
    }
    match trimmed.split_once('\n') {
        Some((key, msg)) => Ok((key.trim_end_matches('\r'), msg.trim_start_matches('\r'))),
        None => Ok((trimmed, "")),
    }
}

fn hex_mac(bytes: impl AsRef<[u8]>) -> String {
    bytes.as_ref().iter().map(|b| format!("{b:02x}")).collect()
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let (key, msg) = split_key_msg(input)?;
    let key_b = key.as_bytes();
    let msg_b = msg.as_bytes();
    let md5 = {
        let mut mac = HmacMd5::new_from_slice(key_b).map_err(|_| "密钥无效".to_string())?;
        mac.update(msg_b);
        hex_mac(mac.finalize().into_bytes())
    };
    let sha1 = {
        let mut mac = HmacSha1::new_from_slice(key_b).map_err(|_| "密钥无效".to_string())?;
        mac.update(msg_b);
        hex_mac(mac.finalize().into_bytes())
    };
    let sha256 = {
        let mut mac = HmacSha256::new_from_slice(key_b).map_err(|_| "密钥无效".to_string())?;
        mac.update(msg_b);
        hex_mac(mac.finalize().into_bytes())
    };
    let sha512 = {
        let mut mac = HmacSha512::new_from_slice(key_b).map_err(|_| "密钥无效".to_string())?;
        mac.update(msg_b);
        hex_mac(mac.finalize().into_bytes())
    };
    Ok(vec![
        ("密钥长度", key.len().to_string()),
        ("消息长度", msg.len().to_string()),
        ("HMAC-MD5", md5),
        ("HMAC-SHA-1", sha1),
        ("HMAC-SHA-256", sha256),
        ("HMAC-SHA-512", sha512),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rfc4231_case() {
        // key = "Jefe", data = "what do ya want for nothing?"
        let rows = convert_all("Jefe\nwhat do ya want for nothing?").unwrap();
        assert_eq!(
            rows.iter().find(|r| r.0 == "HMAC-SHA-256").unwrap().1,
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }
}
