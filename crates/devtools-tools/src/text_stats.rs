//! Text statistics: chars, words, lines, bytes.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::TEXT_STATS,
        name: "Text stats",
        category: Category::Text,
        summary: "统计字符、词、行、字节；输入即更新。",
        keywords: &["count", "words", "chars", "统计", "字数", "行数"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stats {
    pub chars: usize,
    pub chars_no_space: usize,
    pub words: usize,
    pub lines: usize,
    pub bytes: usize,
}

pub fn analyze(input: &str) -> Stats {
    let chars = input.chars().count();
    let chars_no_space = input.chars().filter(|c| !c.is_whitespace()).count();
    let words = input.split_whitespace().count();
    let lines = if input.is_empty() {
        0
    } else {
        input.lines().count()
    };
    Stats {
        chars,
        chars_no_space,
        words,
        lines,
        bytes: input.len(),
    }
}

pub fn rows(input: &str) -> Vec<(&'static str, String)> {
    let s = analyze(input);
    vec![
        ("字符数", s.chars.to_string()),
        ("字符（不含空白）", s.chars_no_space.to_string()),
        ("词数", s.words.to_string()),
        ("行数", s.lines.to_string()),
        ("字节 (UTF-8)", s.bytes.to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        let s = analyze("");
        assert_eq!(s, Stats { chars: 0, chars_no_space: 0, words: 0, lines: 0, bytes: 0 });
    }

    #[test]
    fn sample() {
        let s = analyze("hello 世界\n");
        assert_eq!(s.chars, 9); // h e l l o space 世 界 \n
        assert_eq!(s.words, 2);
        assert_eq!(s.lines, 1);
        assert!(s.bytes > s.chars_no_space);
    }
}
