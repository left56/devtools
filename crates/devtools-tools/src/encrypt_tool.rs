//! Password-based AES-256-GCM text encryption (offline).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::ENCRYPT,
        name: "Encrypt",
        category: Category::Security,
        summary: "AES-256-GCM 文本加解密。第一行口令，其余为明文或密文(Base64)。密钥=SHA256(口令)，仅供学习。",
        keywords: &["encrypt", "decrypt", "aes", "gcm", "加密"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let trimmed = input.trim_end();
    if trimmed.is_empty() {
        return Err("第一行：口令；其余：明文（加密）或 Base64 密文（解密）".into());
    }
    let (pass, rest) = match trimmed.split_once('\n') {
        Some((p, r)) => (p.trim(), r.trim_start_matches('\r')),
        None => return Err("需要至少两行：口令 + 正文/密文".into()),
    };
    if pass.is_empty() || rest.trim().is_empty() {
        return Err("口令与正文都不能为空".into());
    }
    let key = Sha256::digest(pass.as_bytes());
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    // Try decrypt if rest looks like base64 of nonce||ct
    let compact: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
    if let Ok(raw) = base64::engine::general_purpose::STANDARD.decode(&compact)
        && raw.len() > 12
    {
        let (nonce_b, ct) = raw.split_at(12);
        let nonce = Nonce::from_slice(nonce_b);
        if let Ok(plain) = cipher.decrypt(nonce, ct) {
            let text = String::from_utf8_lossy(&plain).into_owned();
            return Ok(vec![
                ("模式", "解密".into()),
                ("明文", text),
                ("说明", "密钥由 SHA-256(口令) 派生；勿用于高安全场景".into()),
            ]);
        }
    }

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ct = cipher
        .encrypt(nonce, rest.as_bytes())
        .map_err(|e| format!("加密失败：{e}"))?;
    let mut out = Vec::with_capacity(12 + ct.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ct);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&out);
    Ok(vec![
        ("模式", "加密".into()),
        ("密文 Base64", b64),
        ("说明", "格式=12字节 nonce || 密文；同一口令可解密。勿把口令当高熵密钥长期用".into()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip() {
        let enc = convert_all("pw\nhello world").unwrap();
        let ct = enc.iter().find(|r| r.0 == "密文 Base64").unwrap().1.clone();
        let dec = convert_all(&format!("pw\n{ct}")).unwrap();
        assert_eq!(dec.iter().find(|r| r.0 == "明文").unwrap().1, "hello world");
    }
}
