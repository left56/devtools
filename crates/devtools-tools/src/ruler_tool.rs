//! CSS / print unit conversions (not a screen overlay ruler).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::RULER,
        name: "Ruler",
        category: Category::Graphics,
        summary: "长度单位换算：px / rem / em / pt / in / cm / mm。屏幕叠尺（量图）未做。默认 1rem=16px。",
        keywords: &["ruler", "px", "rem", "em", "pt", "cm", "单位", "尺"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim().to_ascii_lowercase().replace(' ', "");
    if t.is_empty() {
        return Err("输入如 16px、1.5rem、12pt、2.54cm".into());
    }
    let (value, unit) = split_value_unit(&t)?;
    let px = to_px(value, unit)?;
    Ok(vec![
        ("输入", format!("{value}{unit}")),
        ("px", fmt(px)),
        ("rem (16px)", fmt(px / 16.0)),
        ("em (16px)", fmt(px / 16.0)),
        ("pt", fmt(px * 0.75)),
        ("in", fmt(px / 96.0)),
        ("cm", fmt(px / 96.0 * 2.54)),
        ("mm", fmt(px / 96.0 * 25.4)),
        (
            "说明",
            "基于 CSS 参考像素（96dpi）与 1rem=16px；非屏幕叠尺测量。".into(),
        ),
    ])
}

fn fmt(v: f64) -> String {
    if (v - v.round()).abs() < 1e-9 {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.4}")
    }
}

fn split_value_unit(t: &str) -> Result<(f64, &str), String> {
    let units = ["rem", "em", "px", "pt", "in", "cm", "mm"];
    for u in units {
        if let Some(num) = t.strip_suffix(u) {
            let v: f64 = num
                .parse()
                .map_err(|_| format!("无法解析数字：{num}"))?;
            return Ok((v, u));
        }
    }
    // bare number → px
    let v: f64 = t
        .parse()
        .map_err(|_| "无法识别：需要带单位，如 16px / 1rem".to_string())?;
    Ok((v, "px"))
}

fn to_px(value: f64, unit: &str) -> Result<f64, String> {
    Ok(match unit {
        "px" => value,
        "rem" | "em" => value * 16.0,
        "pt" => value / 0.75,
        "in" => value * 96.0,
        "cm" => value / 2.54 * 96.0,
        "mm" => value / 25.4 * 96.0,
        other => return Err(format!("不支持单位：{other}")),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sixteen_px() {
        let rows = convert_all("16px").unwrap();
        assert_eq!(rows.iter().find(|r| r.0 == "rem (16px)").unwrap().1, "1");
    }

    #[test]
    fn one_inch() {
        let rows = convert_all("1in").unwrap();
        assert_eq!(rows.iter().find(|r| r.0 == "px").unwrap().1, "96");
    }
}
