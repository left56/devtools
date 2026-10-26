//! Rust `regex` crate matching — not PCRE.

use std::fmt;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexError {
    pub message: String,
}

impl fmt::Display for RegexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RegexError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capture {
    pub name: Option<String>,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchInfo {
    pub full: String,
    pub start: usize,
    pub end: usize,
    pub captures: Vec<Capture>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexResult {
    pub matches: Vec<MatchInfo>,
    pub replaced: Option<String>,
    pub syntax_note: &'static str,
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::REGEX,
        name: "Regex",
        category: Category::Text,
        summary: "Rust regex 语法（不是 PCRE）。匹配、捕获组与可选替换。",
        keywords: &["regex", "regexp", "正则", "match", "replace", "rust"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn run(pattern: &str, haystack: &str, replacement: Option<&str>) -> Result<RegexResult, RegexError> {
    if pattern.is_empty() {
        return Err(RegexError {
            message: "模式为空".into(),
        });
    }
    let re = Regex::new(pattern).map_err(|e| RegexError {
        message: format!("编译失败（Rust regex，不是 PCRE）：{e}"),
    })?;

    let mut matches = Vec::new();
    for caps in re.captures_iter(haystack) {
        let full = caps.get(0).expect("full match");
        let mut captures = Vec::new();
        for (i, cap) in caps.iter().enumerate() {
            let Some(m) = cap else { continue };
            let name = re.capture_names().nth(i).flatten().map(str::to_owned);
            captures.push(Capture {
                name,
                text: m.as_str().to_owned(),
                start: m.start(),
                end: m.end(),
            });
        }
        matches.push(MatchInfo {
            full: full.as_str().to_owned(),
            start: full.start(),
            end: full.end(),
            captures,
        });
    }

    let replaced = replacement.map(|r| re.replace_all(haystack, r).into_owned());

    Ok(RegexResult {
        matches,
        replaced,
        syntax_note: "语法为 Rust regex crate，不是 PCRE / JavaScript",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pattern_rejected() {
        assert!(run("", "abc", None).is_err());
    }

    #[test]
    fn typical_captures() {
        let out = run(r"(\w+)@(\w+)", "a@b c@d", None).unwrap();
        assert_eq!(out.matches.len(), 2);
        assert_eq!(out.matches[0].full, "a@b");
        assert_eq!(out.matches[0].captures.len(), 3); // full + 2 groups
        assert!(out.syntax_note.contains("不是 PCRE"));
    }

    #[test]
    fn invalid_pattern() {
        let err = run(r"(", "x", None).unwrap_err();
        assert!(err.message.contains("编译失败"));
    }

    #[test]
    fn replace_optional() {
        let out = run(r"\d+", "a1b22", Some("#")).unwrap();
        assert_eq!(out.replaced.as_deref(), Some("a#b#"));
    }
}
