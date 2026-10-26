//! BIP39 mnemonic generate / validate (learning only).

use crate::quantity;
use bip39::{Language, Mnemonic};
use rand::RngCore;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::BIP39,
        name: "BIP39",
        category: Category::Security,
        summary: "BIP39 助记词生成/校验（英文）。学习用，不是钱包；勿用于真实资产。",
        keywords: &["bip39", "mnemonic", "seed", "助记词"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return generate(12);
    }
    // If input is "12" or "24", generate that many words
    if t == "12" || t == "24" {
        return generate(t.parse().unwrap());
    }
    // validate mnemonic
    match Mnemonic::parse_in_normalized(Language::English, t) {
        Ok(m) => Ok(vec![
            ("校验", "有效".into()),
            ("词数", m.word_count().to_string()),
            ("助记词", m.to_string()),
            ("警告", "仅供学习；不要导入真实钱包或泄露".into()),
        ]),
        Err(e) => Err(format!("无效助记词：{e}")),
    }
}


pub fn generate_many(words: usize, qty: usize) -> Result<Vec<(&'static str, String)>, String> {
    let n = quantity::clamp(qty);
    if n == 1 {
        return generate(words);
    }
    let mut lines = Vec::with_capacity(n);
    for _ in 0..n {
        let rows = generate(words)?;
        let phrase = rows
            .iter()
            .find(|r| r.0 == "助记词")
            .map(|r| r.1.clone())
            .unwrap_or_default();
        lines.push(phrase);
    }
    Ok(vec![
        ("词数", words.to_string()),
        ("数量", n.to_string()),
        ("全部", lines.join("\n")),
        ("警告", "学习/测试专用，不是钱包产品".into()),
    ])
}

pub fn generate(words: usize) -> Result<Vec<(&'static str, String)>, String> {
    let entropy_bytes = match words {
        12 => 16,
        24 => 32,
        _ => 16,
    };
    let mut entropy = vec![0u8; entropy_bytes];
    rand::thread_rng().fill_bytes(&mut entropy);
    let m = Mnemonic::from_entropy_in(Language::English, &entropy)
        .map_err(|e| format!("生成失败：{e}"))?;
    Ok(vec![
        ("词数", m.word_count().to_string()),
        ("助记词", m.to_string()),
        ("警告", "学习/测试专用，不是钱包产品".into()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generate_and_validate() {
        let rows = generate(12).unwrap();
        let phrase = rows.iter().find(|r| r.0 == "助记词").unwrap().1.clone();
        let check = convert_all(&phrase).unwrap();
        assert_eq!(check[0].1, "有效");
        assert_eq!(generate_many(12, 3).unwrap()[2].1.lines().count(), 3);
    }
}
