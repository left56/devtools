//! Identifier case conversion. Not for natural-language pluralization.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseStyle {
    Snake,
    Camel,
    Pascal,
    Kebab,
    Constant,
}

impl CaseStyle {
    pub const ALL: [Self; 5] = [
        Self::Snake,
        Self::Camel,
        Self::Pascal,
        Self::Kebab,
        Self::Constant,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Snake => "snake_case",
            Self::Camel => "camelCase",
            Self::Pascal => "PascalCase",
            Self::Kebab => "kebab-case",
            Self::Constant => "CONSTANT_CASE",
        }
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::CASE,
        name: "Case",
        category: Category::Text,
        summary: "在 snake、camel、kebab、CONSTANT 等标识符风格之间转换。",
        keywords: &[
            "case",
            "snake",
            "camel",
            "kebab",
            "constant",
            "pascal",
            "命名",
            "驼峰",
        ],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Convert an identifier (or a phrase split by spaces / separators) into `style`.
/// Empty input yields an empty string.
pub fn convert(input: &str, style: CaseStyle) -> String {
    let words = split_words(input);
    if words.is_empty() {
        return String::new();
    }
    match style {
        CaseStyle::Snake => words
            .iter()
            .map(|w| w.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join("_"),
        CaseStyle::Kebab => words
            .iter()
            .map(|w| w.to_ascii_lowercase())
            .collect::<Vec<_>>()
            .join("-"),
        CaseStyle::Constant => words
            .iter()
            .map(|w| w.to_ascii_uppercase())
            .collect::<Vec<_>>()
            .join("_"),
        CaseStyle::Camel => {
            let mut parts = words.into_iter();
            let first = parts.next().expect("non-empty").to_ascii_lowercase();
            let rest = parts.map(|w| capitalize(&w)).collect::<String>();
            first + &rest
        }
        CaseStyle::Pascal => words.into_iter().map(|w| capitalize(&w)).collect(),
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => {
            let mut out = String::new();
            out.push(first.to_ascii_uppercase());
            for c in chars {
                out.push(c.to_ascii_lowercase());
            }
            out
        }
    }
}

fn split_words(input: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '_' || ch == '-' || ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
            flush(&mut current, &mut words);
            continue;
        }
        if ch.is_ascii_uppercase() {
            let next_lower = chars
                .peek()
                .copied()
                .is_some_and(|n| n.is_ascii_lowercase());
            let prev_lower = current
                .chars()
                .last()
                .is_some_and(|c| c.is_ascii_lowercase());
            let prev_upper_run = current
                .chars()
                .last()
                .is_some_and(|c| c.is_ascii_uppercase())
                && next_lower;
            if !current.is_empty() && (prev_lower || prev_upper_run) {
                flush(&mut current, &mut words);
            }
            current.push(ch);
            continue;
        }
        if ch.is_ascii_alphanumeric() {
            current.push(ch);
            continue;
        }
        flush(&mut current, &mut words);
    }
    flush(&mut current, &mut words);
    words
}

fn flush(current: &mut String, words: &mut Vec<String>) {
    if !current.is_empty() {
        words.push(std::mem::take(current));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_stays_empty() {
        assert_eq!(convert("", CaseStyle::Snake), "");
        assert_eq!(convert("   ", CaseStyle::Camel), "");
    }

    #[test]
    fn typical_styles() {
        let input = "hello_world";
        assert_eq!(convert(input, CaseStyle::Snake), "hello_world");
        assert_eq!(convert(input, CaseStyle::Camel), "helloWorld");
        assert_eq!(convert(input, CaseStyle::Pascal), "HelloWorld");
        assert_eq!(convert(input, CaseStyle::Kebab), "hello-world");
        assert_eq!(convert(input, CaseStyle::Constant), "HELLO_WORLD");
    }

    #[test]
    fn camel_and_acronyms_round_trip_words() {
        assert_eq!(convert("XMLHttpRequest", CaseStyle::Snake), "xml_http_request");
        assert_eq!(
            convert("xml_http_request", CaseStyle::Pascal),
            "XmlHttpRequest"
        );
        assert_eq!(convert("Hello World", CaseStyle::Kebab), "hello-world");
    }

    #[test]
    fn already_kebab() {
        assert_eq!(convert("foo-bar-baz", CaseStyle::Camel), "fooBarBaz");
    }
}
