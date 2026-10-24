//! Common crontab presets + field explainer (pairs with Cron inspector).

use crate::cron_tool::{self, Zone};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::CRON_PRESETS,
        name: "Cron presets",
        category: Category::Time,
        summary: "常用 cron 模板；也可粘贴 5 段表达式查看字段说明与下次运行。",
        keywords: &["cron", "crontab", "preset", "定时"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const PRESETS: &[(&str, &str)] = &[
    ("每分钟", "* * * * *"),
    ("每小时", "0 * * * *"),
    ("每天 0 点", "0 0 * * *"),
    ("每天 8:30", "30 8 * * *"),
    ("工作日 9 点", "0 9 * * 1-5"),
    ("每周日 0 点", "0 0 * * 0"),
    ("每月 1 号", "0 0 1 * *"),
    ("每 15 分钟", "*/15 * * * *"),
];

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Ok(PRESETS.iter().map(|(n, e)| (*n, (*e).to_string())).collect());
    }
    let fields: Vec<_> = t.split_whitespace().collect();
    if fields.len() == 5 {
        let mut rows = vec![
            ("表达式", fields.join(" ")),
            ("分", fields[0].to_string()),
            ("时", fields[1].to_string()),
            ("日", fields[2].to_string()),
            ("月", fields[3].to_string()),
            ("周", fields[4].to_string()),
        ];
        if let Ok(result) = cron_tool::next_runs(t, Zone::Local, 3) {
            for (i, n) in result.next.iter().enumerate() {
                let label: &'static str = match i {
                    0 => "下次 1",
                    1 => "下次 2",
                    _ => "下次 3",
                };
                rows.push((label, n.clone()));
            }
        }
        return Ok(rows);
    }
    // filter presets by keyword
    let q = t.to_ascii_lowercase();
    let hits: Vec<_> = PRESETS
        .iter()
        .filter(|(n, e)| n.contains(t) || e.contains(&q))
        .collect();
    if hits.is_empty() {
        return Err("未匹配模板；输入 5 段 cron 或留空查看全部".into());
    }
    Ok(hits.into_iter().map(|(n, e)| (*n, (*e).to_string())).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn empty_lists_presets() {
        assert!(convert_all("").unwrap().len() >= 5);
    }
    #[test]
    fn explain_expr() {
        let rows = convert_all("0 9 * * 1-5").unwrap();
        assert_eq!(rows[2].1, "9");
    }
}
