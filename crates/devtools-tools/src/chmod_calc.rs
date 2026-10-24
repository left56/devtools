//! Unix chmod: octal ↔ symbolic (ugo rwx).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::CHMOD,
        name: "Chmod calculator",
        category: Category::Data,
        summary: "权限八进制与符号互转，例如 755 ↔ rwxr-xr-x。",
        keywords: &["chmod", "unix", "permission", "权限", "755"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn octal_to_symbolic(mode: u32) -> Result<String, String> {
    if mode > 0o777 {
        return Err("八进制须为 0–777".into());
    }
    let mut out = String::with_capacity(9);
    for shift in [6, 3, 0] {
        let bits = (mode >> shift) & 0b111;
        out.push(if bits & 0b100 != 0 { 'r' } else { '-' });
        out.push(if bits & 0b010 != 0 { 'w' } else { '-' });
        out.push(if bits & 0b001 != 0 { 'x' } else { '-' });
    }
    Ok(out)
}

pub fn symbolic_to_octal(sym: &str) -> Result<u32, String> {
    let s = sym.trim();
    if s.len() != 9 {
        return Err("符号形式须为 9 个字符，例如 rwxr-xr-x".into());
    }
    let b = s.as_bytes();
    let mut mode = 0u32;
    for (i, expect) in [(0, b'r'), (1, b'w'), (2, b'x')].into_iter().cycle().take(9).enumerate() {
        let ch = b[i];
        let bit = expect.1;
        if ch == bit {
            mode |= 1 << (8 - i);
        } else if ch != b'-' {
            return Err(format!("位置 {} 期望 `{}` 或 `-`", i + 1, bit as char));
        }
    }
    Ok(mode)
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    let (octal, symbolic) = if t.bytes().all(|b| b.is_ascii_digit()) {
        let mode = if t.starts_with('0') {
            u32::from_str_radix(t, 8).map_err(|_| "无效八进制".to_string())?
        } else {
            // Accept decimal-looking 755 as octal digits.
            u32::from_str_radix(t, 8).map_err(|_| "无效八进制（例如 755）".to_string())?
        };
        (mode, octal_to_symbolic(mode)?)
    } else {
        let mode = symbolic_to_octal(t)?;
        (mode, octal_to_symbolic(mode)?)
    };
    Ok(vec![
        ("八进制", format!("{octal:03o}")),
        ("符号", symbolic.clone()),
        ("数字", octal.to_string()),
        ("u", flags(&symbolic.as_bytes()[0..3])),
        ("g", flags(&symbolic.as_bytes()[3..6])),
        ("o", flags(&symbolic.as_bytes()[6..9])),
    ])
}

fn flags(part: &[u8]) -> String {
    let mut v = Vec::new();
    if part[0] == b'r' {
        v.push("read");
    }
    if part[1] == b'w' {
        v.push("write");
    }
    if part[2] == b'x' {
        v.push("execute");
    }
    if v.is_empty() {
        "—".into()
    } else {
        v.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        assert_eq!(octal_to_symbolic(0o755).unwrap(), "rwxr-xr-x");
        assert_eq!(symbolic_to_octal("rwxr-xr-x").unwrap(), 0o755);
        let rows = convert_all("644").unwrap();
        assert_eq!(rows[0].1, "644");
        assert_eq!(rows[1].1, "rw-r--r--");
    }
}
