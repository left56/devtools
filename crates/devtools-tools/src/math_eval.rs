//! Safe math expression evaluator (`fasteval`).
//!
//! Replaces `meval` (pulled ancient `nom` 1.2.4 with future-incompat warnings).

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

fn eval_expr(expr: &str) -> Result<f64, String> {
    // Callback namespace covers the common meval-style functions users expect.
    let mut ns = |name: &str, args: Vec<f64>| -> Option<f64> {
        match (name, args.as_slice()) {
            ("sqrt", [x]) => Some(x.sqrt()),
            ("abs", [x]) => Some(x.abs()),
            ("sin", [x]) => Some(x.sin()),
            ("cos", [x]) => Some(x.cos()),
            ("tan", [x]) => Some(x.tan()),
            ("asin", [x]) => Some(x.asin()),
            ("acos", [x]) => Some(x.acos()),
            ("atan", [x]) => Some(x.atan()),
            ("ln" | "log", [x]) => Some(x.ln()),
            ("log10", [x]) => Some(x.log10()),
            ("exp", [x]) => Some(x.exp()),
            ("floor", [x]) => Some(x.floor()),
            ("ceil", [x]) => Some(x.ceil()),
            ("round", [x]) => Some(x.round()),
            ("min", [a, b]) => Some(a.min(*b)),
            ("max", [a, b]) => Some(a.max(*b)),
            ("pi", []) => Some(std::f64::consts::PI),
            ("e", []) => Some(std::f64::consts::E),
            _ => None,
        }
    };
    fasteval::ez_eval(expr, &mut ns).map_err(|e| format!("无法求值：{e}"))
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入表达式，例如 2+3*sqrt(4)".into());
    }
    let value = eval_expr(t)?;
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
    #[test]
    fn sqrt_fn() {
        let rows = convert_all("sqrt(9)+1").unwrap();
        assert_eq!(rows[0].1, "4");
    }
}
