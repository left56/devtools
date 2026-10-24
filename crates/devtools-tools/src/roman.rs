//! Roman numeral ↔ Arabic integer (1..=3999).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::ROMAN,
        name: "Roman numeral",
        category: Category::Data,
        summary: "阿拉伯数字与罗马数字互转（1–3999）。自动识别方向。",
        keywords: &["roman", "numeral", "罗马", "数字"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const MAP: &[(u32, &str)] = &[
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

pub fn to_roman(n: u32) -> Result<String, String> {
    if !(1..=3999).contains(&n) {
        return Err("范围须为 1–3999".into());
    }
    let mut n = n;
    let mut out = String::new();
    for &(val, sym) in MAP {
        while n >= val {
            out.push_str(sym);
            n -= val;
        }
    }
    Ok(out)
}

pub fn from_roman(input: &str) -> Result<u32, String> {
    let s = input.trim().to_ascii_uppercase();
    if s.is_empty() {
        return Err("输入为空".into());
    }
    let bytes = s.as_bytes();
    let mut i = 0;
    let mut total: u32 = 0;
    while i < bytes.len() {
        let rest = std::str::from_utf8(&bytes[i..]).unwrap();
        let mut matched = false;
        for &(val, sym) in MAP {
            if rest.starts_with(sym) {
                total = total.saturating_add(val);
                i += sym.len();
                matched = true;
                break;
            }
        }
        if !matched {
            return Err(format!("无法解析罗马数字：`{}`", s.chars().nth(i).unwrap_or('?')));
        }
    }
    if total == 0 || total > 3999 {
        return Err("结果超出 1–3999".into());
    }
    // Validate by round-trip
    let again = to_roman(total)?;
    if again != s {
        return Err(format!("非标准写法（规范形式为 {again}）"));
    }
    Ok(total)
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    if t.bytes().all(|b| b.is_ascii_digit()) {
        let n: u32 = t.parse().map_err(|_| "不是有效整数".to_string())?;
        let roman = to_roman(n)?;
        Ok(vec![("阿拉伯数字", n.to_string()), ("罗马数字", roman)])
    } else {
        let n = from_roman(t)?;
        Ok(vec![("阿拉伯数字", n.to_string()), ("罗马数字", to_roman(n)?)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_values() {
        assert_eq!(to_roman(1994).unwrap(), "MCMXCIV");
        assert_eq!(from_roman("MCMXCIV").unwrap(), 1994);
        assert_eq!(to_roman(4).unwrap(), "IV");
        assert!(from_roman("IIII").is_err());
    }

    #[test]
    fn convert_both_ways() {
        let rows = convert_all("42").unwrap();
        assert_eq!(rows[1].1, "XLII");
        let rows = convert_all("XLII").unwrap();
        assert_eq!(rows[0].1, "42");
    }
}
