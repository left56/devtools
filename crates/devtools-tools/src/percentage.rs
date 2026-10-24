//! Percentage helper: X is what % of Y; X% of Y; percent change.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::PERCENTAGE,
        name: "Percentage",
        category: Category::Data,
        summary: "百分比计算：A 是 B 的百分之几、A% 的 B、涨跌幅。输入 `a b`。",
        keywords: &["percent", "percentage", "百分比", "%"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

fn parse_pair(input: &str) -> Result<(f64, f64), String> {
    let parts: Vec<_> = input.split_whitespace().collect();
    if parts.len() != 2 {
        return Err("请输入两个数字，例如 `25 200` 或 `15 80`".into());
    }
    let a: f64 = parts[0]
        .trim_end_matches('%')
        .parse()
        .map_err(|_| format!("无法解析 `{}`", parts[0]))?;
    let b: f64 = parts[1]
        .trim_end_matches('%')
        .parse()
        .map_err(|_| format!("无法解析 `{}`", parts[1]))?;
    Ok((a, b))
}

fn fmt(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{n:.0}")
    } else {
        format!("{n:.6}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let (a, b) = parse_pair(input)?;
    let mut rows = Vec::new();
    if b != 0.0 {
        rows.push(("A 是 B 的", format!("{}%", fmt(a / b * 100.0))));
    } else {
        rows.push(("A 是 B 的", "B 为 0，无法计算".into()));
    }
    rows.push(("A% 的 B", fmt(a / 100.0 * b)));
    if a != 0.0 {
        rows.push(("从 A 到 B 的变化", format!("{}%", fmt((b - a) / a * 100.0))));
    } else {
        rows.push(("从 A 到 B 的变化", "A 为 0，无法计算".into()));
    }
    rows.push(("A + B%", fmt(a * (1.0 + b / 100.0))));
    rows.push(("A − B%", fmt(a * (1.0 - b / 100.0))));
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let rows = convert_all("25 200").unwrap();
        assert_eq!(rows[0].1, "12.5%");
        assert_eq!(rows[1].1, "50");
    }

    #[test]
    fn bad_input() {
        assert!(convert_all("1").is_err());
        assert!(convert_all("").is_err());
    }
}
