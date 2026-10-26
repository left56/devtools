//! bcrypt hash + verify (offline).

use bcrypt::{hash, verify, DEFAULT_COST};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::BCRYPT,
        name: "bcrypt",
        category: Category::Security,
        summary: "bcrypt 哈希与校验。第一行口令；第二行可选已有哈希以校验。成本默认 12。",
        keywords: &["bcrypt", "password", "hash", "口令"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    convert_with_cost(input, 12)
}

pub fn convert_with_cost(input: &str, cost: u32) -> Result<Vec<(&'static str, String)>, String> {
    let mut lines = input.lines().map(str::trim_end);
    let password = lines
        .next()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| "第一行输入口令".to_string())?;
    let existing = lines.next().map(str::trim).filter(|s| !s.is_empty());
    let cost = cost.clamp(4, 15);
    let hashed = hash(password, cost).map_err(|e| format!("哈希失败：{e}"))?;
    let mut rows = vec![
        ("成本", cost.to_string()),
        ("新哈希", hashed.clone()),
    ];
    if let Some(h) = existing {
        let ok = verify(password, h).unwrap_or(false);
        rows.push(("校验", if ok { "匹配".into() } else { "不匹配".into() }));
        rows.push(("对照哈希", h.to_string()));
    } else {
        rows.push(("提示", "第二行可贴已有哈希做校验；空则只生成".into()));
    }
    let _ = DEFAULT_COST;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hash_and_verify() {
        let rows = convert_with_cost("secret", 4).unwrap();
        let h = rows.iter().find(|r| r.0 == "新哈希").unwrap().1.clone();
        let rows2 = convert_with_cost(&format!("secret\n{h}"), 4).unwrap();
        assert_eq!(rows2.iter().find(|r| r.0 == "校验").unwrap().1, "匹配");
        let rows3 = convert_with_cost(&format!("wrong\n{h}"), 4).unwrap();
        assert_eq!(rows3.iter().find(|r| r.0 == "校验").unwrap().1, "不匹配");
    }
}
