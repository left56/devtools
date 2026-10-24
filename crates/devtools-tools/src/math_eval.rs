//! Safe math expression evaluator (meval).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::MATH_EVAL,
        name: "Math evaluator",
        category: Category::Data,
        summary: "安全数学表达式求值（+ - * / ^ ()、sqrt/sin/…）。无任意代码执行。",
        keywords: &["math", "calc", "evaluate", "计算", "表达式"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入表达式，例如 2+3*sqrt(4)".into());
    }
    let value = meval::eval_str(t).map_err(|e| format!("无法求值：{e}"))?;
    let pretty = if value.fract() == 0.0 && value.abs() < 1e15 {
        format!("{value:.0}")
    } else {
        format!("{value}")
    };
    Ok(vec![
        ("结果", pretty),
        ("科学计数", format!("{value:e}")),
        ("表达式", t.to_string()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let rows = convert_all("2+3*4").unwrap();
        assert_eq!(rows[0].1, "14");
    }
    #[test]
    fn bad() {
        assert!(convert_all("2+").is_err());
    }
}
