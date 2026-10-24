//! Integer conversion among bases 2, 8, 10, and 16.
//! Illegal digits report a 0-based character index into the trimmed input.

use std::fmt;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Base(u32);

impl Base {
    pub const BIN: Self = Self(2);
    pub const OCT: Self = Self(8);
    pub const DEC: Self = Self(10);
    pub const HEX: Self = Self(16);
    pub const ALL: [Self; 4] = [Self::BIN, Self::OCT, Self::DEC, Self::HEX];

    pub fn radix(self) -> u32 {
        self.0
    }

    pub fn label(self) -> &'static str {
        match self.0 {
            2 => "2",
            8 => "8",
            10 => "10",
            16 => "16",
            _ => unreachable!("only 2/8/10/16 are constructed"),
        }
    }

    pub fn from_radix(radix: u32) -> Option<Self> {
        matches!(radix, 2 | 8 | 10 | 16).then_some(Self(radix))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumberBaseError {
    pub message: String,
    /// 0-based char index into the trimmed digit body (sign excluded), when known.
    pub char_index: Option<usize>,
}

impl fmt::Display for NumberBaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.char_index {
            Some(index) => write!(f, "第 {} 个数字字符：{}", index + 1, self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for NumberBaseError {}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::NUMBER_BASE,
        name: "Number Base",
        category: Category::Data,
        summary: "在 2/8/10/16 进制之间转换。非法数字会指出字符位置。",
        keywords: &["binary", "hex", "octal", "进制", "二进制", "十六进制"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Convert `input` from `from` into `to`. Accepts an optional leading `+` / `-`
/// and ignores ASCII whitespace and `_` separators inside the digits.
pub fn convert(input: &str, from: Base, to: Base) -> Result<String, NumberBaseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(NumberBaseError {
            message: "输入为空".into(),
            char_index: None,
        });
    }
    let (negative, body) = match trimmed.as_bytes()[0] {
        b'+' => (false, &trimmed[1..]),
        b'-' => (true, &trimmed[1..]),
        _ => (false, trimmed),
    };
    let body = body.trim();
    if body.is_empty() {
        return Err(NumberBaseError {
            message: "缺少数字".into(),
            char_index: None,
        });
    }

    let mut value: u128 = 0;
    let mut digit_index = 0usize;
    for ch in body.chars() {
        if ch == '_' || ch.is_ascii_whitespace() {
            continue;
        }
        let Some(digit) = ch.to_digit(from.radix()) else {
            return Err(NumberBaseError {
                message: format!("`{ch}` 不是 {from_label} 进制合法数字", from_label = from.label()),
                char_index: Some(digit_index),
            });
        };
        value = value
            .checked_mul(u128::from(from.radix()))
            .and_then(|v| v.checked_add(u128::from(digit)))
            .ok_or_else(|| NumberBaseError {
                message: "数字太大".into(),
                char_index: Some(digit_index),
            })?;
        digit_index += 1;
    }
    if digit_index == 0 {
        return Err(NumberBaseError {
            message: "缺少数字".into(),
            char_index: None,
        });
    }

    let mut digits = to_radix(value, to);
    if negative && value != 0 {
        digits.insert(0, '-');
    }
    Ok(digits)
}

fn to_radix(mut value: u128, base: Base) -> String {
    if value == 0 {
        return "0".into();
    }
    let radix = base.radix();
    let mut chars = Vec::new();
    while value > 0 {
        let digit = (value % u128::from(radix)) as u32;
        chars.push(char::from_digit(digit, radix).expect("digit in range"));
        value /= u128::from(radix);
    }
    chars.into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_rejected() {
        assert!(convert("", Base::DEC, Base::HEX).is_err());
    }

    #[test]
    fn typical_conversions() {
        assert_eq!(convert("255", Base::DEC, Base::HEX).unwrap(), "ff");
        assert_eq!(convert("ff", Base::HEX, Base::DEC).unwrap(), "255");
        assert_eq!(convert("11111111", Base::BIN, Base::DEC).unwrap(), "255");
        assert_eq!(convert("377", Base::OCT, Base::DEC).unwrap(), "255");
    }

    #[test]
    fn round_trip_and_separators() {
        let value = convert("1_000", Base::DEC, Base::BIN).unwrap();
        assert_eq!(convert(&value, Base::BIN, Base::DEC).unwrap(), "1000");
        assert_eq!(convert("-10", Base::DEC, Base::HEX).unwrap(), "-a");
    }

    #[test]
    fn illegal_digit_reports_char_index() {
        let error = convert("12A", Base::DEC, Base::HEX).unwrap_err();
        assert_eq!(error.char_index, Some(2));
        assert!(error.message.contains('A'));
    }
}

/// Convert from an auto-detected or explicit base into all common bases at once.
pub fn convert_all(input: &str, from: Base) -> Result<Vec<( &'static str, String)>, NumberBaseError> {
    let trimmed = input.trim();
    // Parse once via convert to DEC, then fan out.
    let dec = convert(trimmed, from, Base::DEC)?;
    Ok(vec![
        ("二进制 (2)", convert(&dec, Base::DEC, Base::BIN)?),
        ("八进制 (8)", convert(&dec, Base::DEC, Base::OCT)?),
        ("十进制 (10)", dec.clone()),
        ("十六进制 (16)", convert(&dec, Base::DEC, Base::HEX)?),
    ])
}

/// Guess base from prefix (`0x`/`0b`/`0o`) or default to decimal.
pub fn detect_base(input: &str) -> Base {
    let t = input.trim().trim_start_matches(['+', '-']).trim();
    let lower = t.to_ascii_lowercase();
    if lower.starts_with("0x") {
        Base::HEX
    } else if lower.starts_with("0b") {
        Base::BIN
    } else if lower.starts_with("0o") {
        Base::OCT
    } else {
        Base::DEC
    }
}

pub fn normalize_input(input: &str) -> (Base, String) {
    let base = detect_base(input);
    let t = input.trim();
    let (negative, rest) = match t.as_bytes().first() {
        Some(b'-') => (true, &t[1..]),
        Some(b'+') => (false, &t[1..]),
        _ => (false, t),
    };
    let rest = rest.trim();
    let lower = rest.to_ascii_lowercase();
    let body = if lower.starts_with("0x") || lower.starts_with("0b") || lower.starts_with("0o") {
        &rest[2..]
    } else {
        rest
    };
    let mut out = String::new();
    if negative {
        out.push('-');
    }
    out.push_str(body.trim());
    (base, out)
}

#[cfg(test)]
mod convert_all_tests {
    use super::*;

    #[test]
    fn all_bases_from_dec() {
        let rows = convert_all("255", Base::DEC).unwrap();
        assert_eq!(rows[2].1, "255");
        assert_eq!(rows[3].1, "ff");
        assert_eq!(rows[0].1, "11111111");
    }

    #[test]
    fn detect_and_normalize() {
        assert_eq!(detect_base("0xff"), Base::HEX);
        assert_eq!(detect_base("0b1010"), Base::BIN);
        let (b, n) = normalize_input("0xFF");
        assert_eq!(b, Base::HEX);
        assert_eq!(n, "FF");
        let rows = convert_all(&n, b).unwrap();
        assert_eq!(rows[2].1, "255");
    }
}
