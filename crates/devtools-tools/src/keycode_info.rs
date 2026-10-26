//! Key name ↔ key code reference (common subset).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::KEYCODE,
        name: "Keycode info",
        category: Category::Web,
        summary: "按键名与 keyCode / code 对照（常用子集）。",
        keywords: &["keycode", "keyboard", "key", "按键"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const TABLE: &[(&str, u16, &str)] = &[
    ("Backspace", 8, "Backspace"),
    ("Tab", 9, "Tab"),
    ("Enter", 13, "Enter"),
    ("Shift", 16, "ShiftLeft"),
    ("Control", 17, "ControlLeft"),
    ("Alt", 18, "AltLeft"),
    ("Escape", 27, "Escape"),
    ("Space", 32, "Space"),
    ("PageUp", 33, "PageUp"),
    ("PageDown", 34, "PageDown"),
    ("End", 35, "End"),
    ("Home", 36, "Home"),
    ("ArrowLeft", 37, "ArrowLeft"),
    ("ArrowUp", 38, "ArrowUp"),
    ("ArrowRight", 39, "ArrowRight"),
    ("ArrowDown", 40, "ArrowDown"),
    ("Delete", 46, "Delete"),
    ("Digit0", 48, "Digit0"),
    ("Digit1", 49, "Digit1"),
    ("KeyA", 65, "KeyA"),
    ("KeyB", 66, "KeyB"),
    ("KeyC", 67, "KeyC"),
    ("KeyZ", 90, "KeyZ"),
    ("F1", 112, "F1"),
    ("F5", 116, "F5"),
    ("F12", 123, "F12"),
    ("Meta", 91, "MetaLeft"),
];

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入按键名（Enter）或数字 keyCode（13）".into());
    }
    if let Ok(code) = t.parse::<u16>() {
        let hits: Vec<_> = TABLE.iter().filter(|(_, c, _)| *c == code).collect();
        if hits.is_empty() {
            return Err(format!("未收录 keyCode {code}"));
        }
        let h = hits[0];
        return Ok(vec![
            ("名称", h.0.to_string()),
            ("keyCode", h.1.to_string()),
            ("code", h.2.to_string()),
        ]);
    }
    let q = t.to_ascii_lowercase();
    let hits: Vec<_> = TABLE
        .iter()
        .filter(|(n, _, c)| n.to_ascii_lowercase().contains(&q) || c.to_ascii_lowercase().contains(&q))
        .collect();
    if hits.is_empty() {
        return Err("没有匹配的按键".into());
    }
    let mut rows = Vec::new();
    for (i, (n, code, c)) in hits.into_iter().take(8).enumerate() {
        let label: &'static str = match i {
            0 => "匹配 1", 1 => "匹配 2", 2 => "匹配 3", 3 => "匹配 4",
            4 => "匹配 5", 5 => "匹配 6", 6 => "匹配 7", _ => "匹配 8",
        };
        rows.push((label, format!("{n} · keyCode={code} · code={c}")));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn by_code_and_name() {
        assert_eq!(convert_all("13").unwrap()[0].1, "Enter");
        assert!(convert_all("arrow").unwrap()[0].1.to_ascii_lowercase().contains("arrow"));
    }
}
