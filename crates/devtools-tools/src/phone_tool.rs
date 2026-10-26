//! Lightweight phone parse / format (E.164-ish heuristics; not libphonenumber).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::PHONE,
        name: "Phone parse",
        category: Category::Text,
        summary: "轻量电话号码整理：去杂质、E.164 猜测。非完整 libphonenumber。",
        keywords: &["phone", "tel", "e164", "电话", "手机"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    let digits: String = t.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect();
    let raw_digits: String = t.chars().filter(|c| c.is_ascii_digit()).collect();
    if raw_digits.len() < 7 {
        return Err("有效数字过少".into());
    }
    let e164 = if digits.starts_with('+') {
        format!("+{}", digits.chars().filter(|c| c.is_ascii_digit()).collect::<String>())
    } else if raw_digits.len() == 11 && raw_digits.starts_with('1') && looks_cn_mobile(&raw_digits[1..]) {
        // 1xxxxxxxxxx CN mobile with leading country? uncommon
        format!("+{raw_digits}")
    } else if raw_digits.len() == 11 && looks_cn_mobile(&raw_digits) {
        format!("+86{raw_digits}")
    } else if raw_digits.len() == 10 && raw_digits.starts_with('1') {
        // US-ish
        format!("+1{raw_digits}")
    } else {
        format!("+{raw_digits}")
    };
    let national = if e164.starts_with("+86") && e164.len() == 14 {
        let n = &e164[3..];
        format!("{} {} {} {}", &n[0..3], &n[3..7], &n[7..11], "")
            .trim()
            .to_string()
    } else if e164.starts_with("+1") && e164.len() == 12 {
        let n = &e164[2..];
        format!("({}) {}-{}", &n[0..3], &n[3..6], &n[6..10])
    } else {
        e164.clone()
    };
    Ok(vec![
        ("数字", raw_digits),
        ("E.164（猜测）", e164),
        ("本地可读（猜测）", national),
        ("说明", "启发式整理，国家码可能不准；复杂号码请用专业库".into()),
    ])
}

fn looks_cn_mobile(s: &str) -> bool {
    s.len() == 11 && s.starts_with('1') && s.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cn_mobile() {
        let rows = convert_all("138-0013-8000").unwrap();
        assert_eq!(rows[1].1, "+8613800138000");
    }
}
