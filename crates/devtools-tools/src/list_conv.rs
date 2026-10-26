//! List converter: split, sort, reverse, dedupe, rejoin.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::LIST_CONV,
        name: "List converter",
        category: Category::Text,
        summary: "列表转换：按行/逗号拆分，排序、反转、去重，并给出多种连接结果。",
        keywords: &["list", "sort", "dedupe", "join", "列表", "去重"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

fn split_items(input: &str) -> Vec<String> {
    let t = input.trim();
    if t.is_empty() {
        return Vec::new();
    }
    if t.contains('\n') {
        t.lines()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    } else {
        t.split([',', ';', '|'])
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect()
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let items = split_items(input);
    if items.is_empty() {
        return Err("输入为空".into());
    }
    let mut sorted = items.clone();
    sorted.sort();
    let mut deduped = items.clone();
    let mut seen = std::collections::BTreeSet::new();
    deduped.retain(|x| seen.insert(x.clone()));
    let mut rev = items.clone();
    rev.reverse();
    Ok(vec![
        ("条目数", items.len().to_string()),
        ("去重后", deduped.len().to_string()),
        ("逗号连接", items.join(", ")),
        ("换行连接", items.join("\n")),
        ("排序（换行）", sorted.join("\n")),
        ("反转（换行）", rev.join("\n")),
        ("去重（换行）", deduped.join("\n")),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comma_and_dedupe() {
        let rows = convert_all("b, a, b, c").unwrap();
        assert_eq!(rows[0].1, "4");
        assert_eq!(rows[1].1, "3");
    }
}
