//! Light string obfuscator (mask middle).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::OBFUSCATE,
        name: "String obfuscator",
        category: Category::Text,
        summary: "遮罩字符串中间部分，便于截图分享。",
        keywords: &["obfuscate", "mask", "redact", "遮罩", "脱敏"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    if input.is_empty() {
        return Err("输入为空".into());
    }
    Ok(vec![
        ("保留两端 1", mask(input, 1)),
        ("保留两端 2", mask(input, 2)),
        ("保留两端 4", mask(input, 4)),
        ("全星号", "*".repeat(input.chars().count())),
    ])
}

fn mask(s: &str, keep: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    let n = chars.len();
    if n <= keep * 2 {
        return "*".repeat(n);
    }
    let mut out = String::new();
    for (i, ch) in chars.into_iter().enumerate() {
        if i < keep || i >= n - keep {
            out.push(ch);
        } else {
            out.push('*');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mask_emailish() {
        let rows = convert_all("hello@example.com").unwrap();
        assert!(rows[1].1.starts_with("he"));
        assert!(rows[1].1.ends_with("om"));
        assert!(rows[1].1.contains('*'));
    }
}
