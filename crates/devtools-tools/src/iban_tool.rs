//! IBAN validator and parser (mod-97).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::IBAN,
        name: "IBAN",
        category: Category::Data,
        summary: "IBAN 校验与解析（ISO 13616 mod-97），不联网查银行。",
        keywords: &["iban", "bank", "sepa", "银行账号"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let raw: String = input
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if raw.len() < 15 || raw.len() > 34 {
        return Err("IBAN 长度应在 15–34".into());
    }
    if !raw.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err("含非法字符".into());
    }
    let country = &raw[0..2];
    if !country.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err("国家码须为字母".into());
    }
    let check = &raw[2..4];
    let bban = &raw[4..];
    let valid = mod97(&raw)?;
    let grouped = group4(&raw);
    Ok(vec![
        ("规范化", raw.clone()),
        ("分组", grouped),
        ("国家", country.to_string()),
        ("校验位", check.to_string()),
        ("BBAN", bban.to_string()),
        ("mod-97", if valid { "通过".into() } else { "失败".into() }),
    ])
}

fn group4(s: &str) -> String {
    s.as_bytes()
        .chunks(4)
        .map(|c| std::str::from_utf8(c).unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

fn mod97(iban: &str) -> Result<bool, String> {
    let rearranged = format!("{}{}", &iban[4..], &iban[..4]);
    let mut expanded = String::new();
    for ch in rearranged.chars() {
        if ch.is_ascii_digit() {
            expanded.push(ch);
        } else if ch.is_ascii_alphabetic() {
            expanded.push_str(&(ch as u32 - 55).to_string());
        } else {
            return Err("非法字符".into());
        }
    }
    // Running mod 97 on digit string
    let mut rem = 0u32;
    for ch in expanded.chars() {
        let d = ch.to_digit(10).unwrap();
        rem = (rem * 10 + d) % 97;
    }
    Ok(rem == 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn valid_iban() {
        let rows = convert_all("GB82 WEST 1234 5698 7654 32").unwrap();
        assert_eq!(rows.iter().find(|r| r.0 == "mod-97").unwrap().1, "通过");
        assert_eq!(rows.iter().find(|r| r.0 == "国家").unwrap().1, "GB");
    }
}
