//! Minimal banner-style ASCII text (tiny 3-row font).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::ASCII_DRAW,
        name: "ASCII text",
        category: Category::Text,
        summary: "简易 ASCII 横幅（内置极简 3 行字模，非完整 FIGlet）。",
        keywords: &["ascii", "banner", "figlet", "艺术字"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Very small 3-row glyphs for A-Z, 0-9, space.
fn glyph(ch: char) -> [&'static str; 3] {
    match ch.to_ascii_uppercase() {
        'A' => [" _ ", "|_|", "| |"],
        'B' => [" _ ", "|_)", "|_)"],
        'C' => [" _ ", "|  ", "|_ "],
        'D' => [" _ ", "| \\", "|_/"],
        'E' => [" _ ", "|_ ", "|_ "],
        'F' => [" _ ", "|_ ", "|  "],
        'G' => [" _ ", "| _", "|_|"],
        'H' => ["   ", "|_|", "| |"],
        'I' => ["___", " | ", "_|_"],
        'J' => ["___", " | ", "_| "],
        'K' => ["   ", "|/ ", "|\\ "],
        'L' => ["   ", "|  ", "|_ "],
        'M' => ["   ", "|\\/|", "|  |"],
        'N' => ["   ", "|\\|", "| |"],
        'O' => [" _ ", "| |", "|_|"],
        'P' => [" _ ", "|_|", "|  "],
        'Q' => [" _ ", "| |", "|_\\"],
        'R' => [" _ ", "|_|", "| \\"],
        'S' => [" _ ", "|_ ", " _|"],
        'T' => ["___", " | ", " | "],
        'U' => ["   ", "| |", "|_|"],
        'V' => ["   ", "| |", " v "],
        'W' => ["   ", "| |", "|/\\|"],
        'X' => ["   ", "\\/ ", "/\\ "],
        'Y' => ["   ", "\\/ ", " | "],
        'Z' => ["___", " / ", "/__"],
        '0' => [" _ ", "| |", "|_|"],
        '1' => ["   ", "/| ", "_|_"],
        '2' => [" _ ", " _|", "|_ "],
        '3' => [" _ ", " _|", " _|"],
        '4' => ["   ", "|_|", "  |"],
        '5' => [" _ ", "|_ ", " _|"],
        '6' => [" _ ", "|_ ", "|_|"],
        '7' => ["___", "  /", " / "],
        '8' => [" _ ", "|_|", "|_|"],
        '9' => [" _ ", "|_|", " _|"],
        ' ' => ["   ", "   ", "   "],
        '!' => [" | ", " | ", " . "],
        '?' => [" _ ", " _|", " . "],
        _ => ["???", "???", "???"],
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入要绘制的短文本（建议 ≤ 16 字符）".into());
    }
    let chars: Vec<char> = t.chars().take(24).collect();
    let mut rows = [String::new(), String::new(), String::new()];
    for ch in chars {
        let g = glyph(ch);
        for i in 0..3 {
            if !rows[i].is_empty() {
                rows[i].push(' ');
            }
            rows[i].push_str(g[i]);
        }
    }
    let banner = rows.join("\n");
    let block = format!("```\n{banner}\n```");
    Ok(vec![
        ("横幅", banner),
        ("Markdown 代码块", block),
        ("说明", "极简字模；完整 FIGlet 字体未内置".into()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn draws_hi() {
        let rows = convert_all("HI").unwrap();
        assert_eq!(rows[0].1.lines().count(), 3);
    }
}
