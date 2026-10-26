//! Offline password strength heuristic.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::PASSWORD_STRENGTH,
        name: "Password strength",
        category: Category::Security,
        summary: "口令强度启发式评分与建议。不上传、不查泄露库。",
        keywords: &["password", "strength", "口令", "强度"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    if input.is_empty() {
        return Err("输入为空".into());
    }
    // Allow trailing newline from paste but score without final \n only if whole input is password
    let pw = input.trim_end_matches('\n').trim_end_matches('\r');
    if pw.is_empty() {
        return Err("输入为空".into());
    }
    let mut score = 0i32;
    let mut tips = Vec::new();
    let len = pw.chars().count();
    if len >= 12 {
        score += 2;
    } else if len >= 8 {
        score += 1;
    } else {
        tips.push("长度至少 12 更稳妥");
    }
    let has_lower = pw.chars().any(|c| c.is_ascii_lowercase());
    let has_upper = pw.chars().any(|c| c.is_ascii_uppercase());
    let has_digit = pw.chars().any(|c| c.is_ascii_digit());
    let has_sym = pw.chars().any(|c| !c.is_ascii_alphanumeric());
    for (ok, tip, pts) in [
        (has_lower, "加入小写字母", 1),
        (has_upper, "加入大写字母", 1),
        (has_digit, "加入数字", 1),
        (has_sym, "加入符号", 2),
    ] {
        if ok {
            score += pts;
        } else {
            tips.push(tip);
        }
    }
    let common = ["password", "123456", "qwerty", "admin", "letmein", "welcome"];
    if common.iter().any(|c| pw.to_ascii_lowercase().contains(c)) {
        score -= 2;
        tips.push("避免常见口令片段");
    }
    if has_sequence(pw) {
        score -= 1;
        tips.push("避免连续键盘序列（如 abc、123）");
    }
    score = score.clamp(0, 8);
    let label = match score {
        0..=2 => "很弱",
        3..=4 => "弱",
        5..=6 => "中等",
        _ => "强",
    };
    let mut rows = vec![
        ("长度", len.to_string()),
        ("评分", format!("{score}/8")),
        ("等级", label.into()),
        (
            "字符类",
            format!(
                "小写={} 大写={} 数字={} 符号={}",
                yn(has_lower),
                yn(has_upper),
                yn(has_digit),
                yn(has_sym)
            ),
        ),
    ];
    if tips.is_empty() {
        rows.push(("建议", "看起来不错；仍建议使用密码管理器".into()));
    } else {
        rows.push(("建议", tips.join("；")));
    }
    Ok(rows)
}

fn yn(b: bool) -> &'static str {
    if b { "有" } else { "无" }
}

fn has_sequence(s: &str) -> bool {
    let b: Vec<u8> = s.bytes().map(|c| c.to_ascii_lowercase()).collect();
    b.windows(3).any(|w| {
        (w[0] + 1 == w[1] && w[1] + 1 == w[2]) || (w[0] == w[1] && w[1] == w[2])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn weak_and_strong() {
        let weak = convert_all("password").unwrap();
        assert!(weak[2].1.contains('弱') || weak[2].1.contains("很弱"));
        let strong = convert_all("Tr0ub4dor&3xQ!").unwrap();
        assert!(strong[2].1.contains('强') || strong[2].1.contains('中'));
    }
}
