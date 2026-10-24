//! Lorem ipsum paragraph generator. Explicit generate; offline word list.

use crate::quantity;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

const WORDS: &[&str] = &[
    "lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit",
    "sed", "do", "eiusmod", "tempor", "incididunt", "ut", "labore", "et", "dolore",
    "magna", "aliqua", "ut", "enim", "ad", "minim", "veniam", "quis", "nostrud",
    "exercitation", "ullamco", "laboris", "nisi", "ut", "aliquip", "ex", "ea",
    "commodo", "consequat", "duis", "aute", "irure", "dolor", "in", "reprehenderit",
    "in", "voluptate", "velit", "esse", "cillum", "dolore", "eu", "fugiat", "nulla",
    "pariatur", "excepteur", "sint", "occaecat", "cupidatat", "non", "proident",
    "sunt", "in", "culpa", "qui", "officia", "deserunt", "mollit", "anim", "id",
    "est", "laborum",
];

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::LOREM,
        name: "Lorem ipsum",
        category: Category::Generators,
        summary: "生成占位拉丁文。可指定段落数与份数。",
        keywords: &["lorem", "ipsum", "placeholder", "占位", "假文"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Generate `qty` independent blocks (each with `paragraphs` paragraphs).
pub fn generate_many(paragraphs: usize, qty: usize) -> String {
    let n = quantity::clamp(qty);
    (0..n)
        .map(|_| generate(paragraphs))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Generate `paragraphs` paragraphs (clamped 1..=20). Deterministic from seed counter.
pub fn generate(paragraphs: usize) -> String {
    let n = paragraphs.clamp(1, 20);
    let mut out = String::new();
    let mut idx = 0usize;
    for p in 0..n {
        if p > 0 {
            out.push_str("\n\n");
        }
        let words_in_para = 40 + (p % 20);
        for w in 0..words_in_para {
            if w > 0 {
                out.push(' ');
            }
            let word = WORDS[idx % WORDS.len()];
            if w == 0 {
                let mut chars = word.chars();
                if let Some(c) = chars.next() {
                    out.extend(c.to_uppercase());
                    out.push_str(chars.as_str());
                }
            } else {
                out.push_str(word);
            }
            idx += 1;
        }
        out.push('.');
    }
    out
}

/// Parse a paragraph count from input; default 1.
pub fn parse_count(input: &str) -> usize {
    let t = input.trim();
    if t.is_empty() {
        return 1;
    }
    t.parse::<usize>().unwrap_or(1).clamp(1, 20)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_paragraph() {
        let text = generate(1);
        assert!(text.starts_with("Lorem"));
        assert!(text.ends_with('.'));
        assert!(!text.contains("\n\n"));
    }

    #[test]
    fn generate_many_count() {
        assert_eq!(generate_many(1, 3).split("\n\n").count(), 3);
    }

    #[test]
    fn multi_and_clamp() {
        assert_eq!(generate(3).matches("\n\n").count(), 2);
        assert_eq!(generate(100).matches("\n\n").count(), 19);
        assert_eq!(parse_count(""), 1);
        assert_eq!(parse_count("5"), 5);
    }
}
