//! Static regex cheatsheet (Rust/JS common).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::REGEX_MEMO,
        name: "Regex memo",
        category: Category::Text,
        summary: "正则速查表。输入关键词过滤；与「正则测试」工具分开。",
        keywords: &["regex", "regexp", "memo", "速查", "正则"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const ENTRIES: &[(&str, &str)] = &[
    (".", "任意字符（默认不含换行）"),
    ("\\d \\w \\s", "数字 / 单词字符 / 空白"),
    ("\\D \\W \\S", "非数字 / 非单词 / 非空白"),
    ("^ $", "行首 / 行尾"),
    ("\\b \\B", "单词边界 / 非边界"),
    ("*", "0 次或多次（贪婪）"),
    ("+", "1 次或多次"),
    ("?", "0 或 1 次；也可使量词惰性"),
    ("{n}", "恰好 n 次"),
    ("{n,m}", "n 到 m 次"),
    ("|", "或"),
    ("[]", "字符类，如 [a-z0-9]"),
    ("[^]", "取反字符类"),
    ("()", "捕获组"),
    ("(?:)", "非捕获组"),
    ("(?=)", "正向先行断言"),
    ("(?!)", "负向先行断言"),
    ("\\1", "反向引用第 1 组"),
    ("email-ish", r"[^@\s]+@[^@\s]+\.[^@\s]+"),
    ("url-ish", r"https?://\S+"),
    ("ipv4-ish", r"\b(?:\d{1,3}\.){3}\d{1,3}\b"),
    ("flags", "i 忽略大小写 · m 多行 · s 点匹配换行（视引擎）"),
];

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let q = input.trim().to_ascii_lowercase();
    let hits: Vec<_> = ENTRIES
        .iter()
        .filter(|(k, v)| q.is_empty() || k.to_ascii_lowercase().contains(&q) || v.to_ascii_lowercase().contains(&q))
        .collect();
    if hits.is_empty() {
        return Err("没有匹配的条目".into());
    }
    Ok(hits
        .into_iter()
        .take(30)
        .map(|(k, v)| (*k, (*v).to_string()))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn digit_entry() {
        let rows = convert_all("digit").unwrap_or_else(|_| convert_all("\\d").unwrap());
        assert!(!rows.is_empty());
    }
}
