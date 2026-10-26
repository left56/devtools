//! Password-based AES-256-GCM text encryption (offline).

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::Engine;
use rand::RngCore;
use sha2::{Digest, Sha256};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptMode {
    Encrypt,
    Decrypt,
}

impl EncryptMode {
    pub const ALL: [EncryptMode; 2] = [EncryptMode::Encrypt, EncryptMode::Decrypt];

    pub fn label(self) -> &'static str {
        match self {
            Self::Encrypt => "加密",
            Self::Decrypt => "解密",
        }
    }

    pub fn default_mode() -> Self {
        Self::Encrypt
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::ENCRYPT,
        name: "Encrypt",
        category: Category::Security,
        summary: "AES-256-GCM 文本加解密。口令独立填写；模式：加密 | 解密。密钥=SHA256(口令)，仅供学习。",
        keywords: &["encrypt", "decrypt", "aes", "gcm", "加密", "解密"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Legacy: first line password, rest body. Prefer [`convert_mode`].
pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let trimmed = input.trim_end();
    if trimmed.is_empty() {
        return Err("需要口令与正文".into());
    }
    let (pass, rest) = match trimmed.split_once('\n') {
        Some((p, r)) => (p.trim(), r.trim_start_matches('\r')),
        None => return Err("需要口令与正文（两行）或使用独立口令栏".into()),
    };
    let compact: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
    let mode = if compact.len() > 16
        && base64::engine::general_purpose::STANDARD
            .decode(&compact)
            .map(|b| b.len() > 12)
            .unwrap_or(false)
    {
        EncryptMode::Decrypt
    } else {
        EncryptMode::Encrypt
    };
    convert_mode(rest, pass, mode)
}

pub fn convert_mode(
    body: &str,
    password: &str,
    mode: EncryptMode,
) -> Result<Vec<(&'static str, String)>, String> {
    let pass = password.trim();
    let rest = body.trim_start_matches('\r');
    if pass.is_empty() {
        return Err("口令不能为空".into());
    }
    if rest.trim().is_empty() {
        return Err(match mode {
            EncryptMode::Encrypt => "明文不能为空".into(),
            EncryptMode::Decrypt => "密文不能为空".into(),
        });
    }
    let key = Sha256::digest(pass.as_bytes());
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;

    match mode {
        EncryptMode::Decrypt => {
            let compact: String = rest.chars().filter(|c| !c.is_whitespace()).collect();
            let raw = base64::engine::general_purpose::STANDARD
                .decode(&compact)
                .map_err(|e| format!("密文不是合法 Base64：{e}"))?;
            if raw.len() <= 12 {
                return Err("密文太短（需要 nonce + 密文）".into());
            }
            let (nonce_b, ct) = raw.split_at(12);
            let nonce = Nonce::from_slice(nonce_b);
            let plain = cipher
                .decrypt(nonce, ct)
                .map_err(|_| "解密失败（口令错误或密文损坏）".to_string())?;
            let text = String::from_utf8_lossy(&plain).into_owned();
            Ok(vec![
                ("明文", text),
                (
                    "说明",
                    "密钥由 SHA-256(口令) 派生；勿用于高安全场景".into(),
                ),
            ])
        }
        EncryptMode::Encrypt => {
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
                ("密文 Base64", b64),
                (
                    "说明",
                    "格式=12字节 nonce || 密文；同一口令可解密。勿把口令当高熵密钥长期用".into(),
                ),
            ])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn round_trip_mode() {
        let enc = convert_mode("hello world", "pw", EncryptMode::Encrypt).unwrap();
        let ct = enc.iter().find(|r| r.0 == "密文 Base64").unwrap().1.clone();
        let dec = convert_mode(&ct, "pw", EncryptMode::Decrypt).unwrap();
        assert_eq!(dec.iter().find(|r| r.0 == "明文").unwrap().1, "hello world");
    }

    #[test]
    fn legacy_two_line_still_works() {
        let enc = convert_all("pw\nhello world").unwrap();
        let ct = enc.iter().find(|r| r.0 == "密文 Base64").unwrap().1.clone();
        let dec = convert_all(&format!("pw\n{ct}")).unwrap();
        assert_eq!(dec.iter().find(|r| r.0 == "明文").unwrap().1, "hello world");
    }
}
