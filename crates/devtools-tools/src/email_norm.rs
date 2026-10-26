//! Email address normalizer / splitter.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::EMAIL_NORM,
        name: "Email normalizer",
        category: Category::Text,
        summary: "规范化邮箱：小写、去掉 Gmail 点号与 +tag，拆分本地/域名。",
        keywords: &["email", "gmail", "normalize", "邮箱"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    let lower = t.to_ascii_lowercase();
    let (local, domain) = lower
        .split_once('@')
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .ok_or_else(|| "缺少 @".to_string())?;
    if local.is_empty() || domain.is_empty() || !domain.contains('.') {
        return Err("邮箱格式无效".into());
    }
    let plus_stripped = local.split('+').next().unwrap_or(&local).to_string();
    let gmail_norm = if domain == "gmail.com" || domain == "googlemail.com" {
        plus_stripped.replace('.', "")
    } else {
        plus_stripped.clone()
    };
    let normalized = format!("{gmail_norm}@{domain}");
    Ok(vec![
        ("原文小写", lower),
        ("本地部分", local),
        ("域名", domain),
        ("规范化", normalized),
        ("去 +tag 本地", plus_stripped),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gmail_dots() {
        let rows = convert_all("John.Doe+news@Gmail.com").unwrap();
        assert_eq!(
            rows.iter().find(|r| r.0 == "规范化").unwrap().1,
            "johndoe@gmail.com"
        );
    }
}
