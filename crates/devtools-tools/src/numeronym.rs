//! Numeronym generator (i18n → i18n, kubernetes → k8s).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::NUMERONYM,
        name: "Numeronym",
        category: Category::Text,
        summary: "数字缩写：internationalization → i18n，kubernetes → k8s。",
        keywords: &["numeronym", "i18n", "a11y", "k8s", "缩写"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert(input: &str) -> Result<String, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    let words: Vec<_> = t.split_whitespace().map(numeronym_word).collect();
    Ok(words.join(" "))
}

fn numeronym_word(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    if chars.len() < 3 {
        return word.to_string();
    }
    format!("{}{}{}", chars[0], chars.len() - 2, chars[chars.len() - 1])
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    Ok(vec![("Numeronym", convert(input)?)])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn known() {
        assert_eq!(convert("internationalization").unwrap(), "i18n");
        assert_eq!(convert("kubernetes").unwrap(), "k8s");
        assert_eq!(convert("accessibility").unwrap(), "a11y");
    }
}
