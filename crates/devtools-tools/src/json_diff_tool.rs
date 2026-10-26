//! Structural JSON diff (paths that differ).

use serde_json::Value;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::JSON_DIFF,
        name: "JSON diff",
        category: Category::Data,
        summary: "结构化 JSON 差异：用 --- 分隔左右两份 JSON。",
        keywords: &["json", "diff", "compare", "对比"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空：左 JSON --- 右 JSON".into());
    }
    let (left_s, right_s) = split_pair(t)?;
    let left: Value = serde_json::from_str(left_s).map_err(|e| format!("左侧 JSON：{e}"))?;
    let right: Value = serde_json::from_str(right_s).map_err(|e| format!("右侧 JSON：{e}"))?;
    let mut diffs = Vec::new();
    walk("", &left, &right, &mut diffs);
    if diffs.is_empty() {
        return Ok(vec![("结果", "相同".into()), ("差异数", "0".into())]);
    }
    let mut rows = vec![("差异数", diffs.len().to_string())];
    for (i, line) in diffs.iter().take(40).enumerate() {
        let label: &'static str = match i {
            0 => "差异 1", 1 => "差异 2", 2 => "差异 3", 3 => "差异 4", 4 => "差异 5",
            5 => "差异 6", 6 => "差异 7", 7 => "差异 8", 8 => "差异 9", 9 => "差异 10",
            _ => "差异+",
        };
        if i < 10 {
            rows.push((label, line.clone()));
        } else {
            rows.push(("差异+", line.clone()));
            break;
        }
    }
    if diffs.len() > 10 {
        rows.push(("其余", format!("还有 {} 处未列出", diffs.len() - 10)));
    }
    Ok(rows)
}

fn split_pair(t: &str) -> Result<(&str, &str), String> {
    for sep in ["\n---\n", "\n--\n", "\n===\n"] {
        if let Some((a, b)) = t.split_once(sep) {
            return Ok((a.trim(), b.trim()));
        }
    }
    Err("请用单独一行的 --- 分隔左右两份 JSON".into())
}

fn walk(path: &str, left: &Value, right: &Value, out: &mut Vec<String>) {
    if left == right {
        return;
    }
    match (left, right) {
        (Value::Object(a), Value::Object(b)) => {
            let mut keys: Vec<_> = a.keys().chain(b.keys()).collect();
            keys.sort();
            keys.dedup();
            for k in keys {
                let p = if path.is_empty() {
                    k.clone()
                } else {
                    format!("{path}.{k}")
                };
                match (a.get(k), b.get(k)) {
                    (Some(lv), Some(rv)) => walk(&p, lv, rv, out),
                    (Some(lv), None) => out.push(format!("{p}: 仅左侧 = {lv}")),
                    (None, Some(rv)) => out.push(format!("{p}: 仅右侧 = {rv}")),
                    _ => {}
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            let n = a.len().max(b.len());
            for i in 0..n {
                let p = format!("{path}[{i}]");
                match (a.get(i), b.get(i)) {
                    (Some(lv), Some(rv)) => walk(&p, lv, rv, out),
                    (Some(lv), None) => out.push(format!("{p}: 仅左侧 = {lv}")),
                    (None, Some(rv)) => out.push(format!("{p}: 仅右侧 = {rv}")),
                    _ => {}
                }
            }
        }
        _ => out.push(format!("{path}: {left} → {right}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn finds_change() {
        let rows = convert_all("{\"a\":1}\n---\n{\"a\":2}").unwrap();
        assert_eq!(rows[0].1, "1");
        assert!(rows[1].1.contains('a'));
    }
}
