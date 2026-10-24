//! RSA key pair generator (PEM).

use crate::quantity;
use rand::rngs::OsRng;
use rsa::pkcs1::{EncodeRsaPrivateKey, EncodeRsaPublicKey, LineEnding};
use rsa::{RsaPrivateKey, RsaPublicKey};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::RSA_KEYGEN,
        name: "RSA keygen",
        category: Category::Security,
        summary: "生成 RSA PEM 密钥对（默认 2048）。点击生成；仅供开发调试。",
        keywords: &["rsa", "pem", "keypair", "密钥"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}


/// Multiple keypairs (clamped 1..=5 — RSA is expensive).
pub fn generate_many(bits: usize, qty: usize) -> Result<Vec<(&'static str, String)>, String> {
    let n = quantity::clamp(qty).min(5);
    if n == 1 {
        return generate(bits);
    }
    let mut blocks = Vec::with_capacity(n);
    for i in 1..=n {
        let rows = generate(bits)?;
        let priv_pem = rows
            .iter()
            .find(|r| r.0.contains("私钥") || r.0.to_ascii_lowercase().contains("private"))
            .map(|r| r.1.clone())
            .or_else(|| rows.first().map(|r| r.1.clone()))
            .unwrap_or_default();
        blocks.push(format!("# {i}\n{priv_pem}"));
    }
    Ok(vec![
        ("数量", n.to_string()),
        ("位数", bits.to_string()),
        ("全部私钥 PEM", blocks.join("\n\n")),
        ("警告", "仅供开发调试".into()),
    ])
}

pub fn generate(bits: usize) -> Result<Vec<(&'static str, String)>, String> {
    let bits = match bits {
        1024 | 2048 | 4096 => bits,
        _ => 2048,
    };
    let mut rng = OsRng;
    let private = RsaPrivateKey::new(&mut rng, bits).map_err(|e| format!("生成失败：{e}"))?;
    let public = RsaPublicKey::from(&private);
    let priv_pem = private
        .to_pkcs1_pem(LineEnding::LF)
        .map_err(|e| e.to_string())?
        .to_string();
    let pub_pem = public
        .to_pkcs1_pem(LineEnding::LF)
        .map_err(|e| e.to_string())?
        .to_string();
    Ok(vec![
        ("位数", bits.to_string()),
        ("公钥 PEM", pub_pem),
        ("私钥 PEM", priv_pem),
        ("警告", "私钥勿提交仓库；生产请用专用 KMS/openssl 流程".into()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gen_1024_for_speed() {
        let rows = generate(1024).unwrap();
        assert!(rows[1].1.contains("BEGIN RSA PUBLIC KEY"));
        assert!(rows[2].1.contains("BEGIN RSA PRIVATE KEY"));
    }
}
