//! ASCII slugify: lowercase, non-alnum → hyphen, collapse/trim hyphens.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::SLUGIFY,
        name: "Slugify",
        category: Category::Text,
        summary: "一次输入给出 Slug 与规范化变体。",
        keywords: &["slug", "url", "slugify", "kebab", "路径"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("输入为空".into());
    }
    let slug = slugify(trimmed);
    let lower = trimmed.to_ascii_lowercase();
    let collapsed = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
    Ok(vec![
        ("Slug", slug.clone()),
        ("小写原文", lower),
        ("空白折叠", collapsed),
        ("原文", trimmed.to_string()),
    ])
}

pub fn slugify(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_hyphen = true; // trim leading
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_hyphen = false;
        } else if !prev_hyphen {
            out.push('-');
            prev_hyphen = true;
        }
    }
    if out.ends_with('-') {
        out.pop();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(slugify("Hello World!"), "hello-world");
        assert_eq!(slugify("  Foo   Bar  "), "foo-bar");
        assert_eq!(slugify("déjà-vu"), "d-j-vu"); // non-ascii letters dropped
        assert_eq!(slugify(""), "");
        assert_eq!(slugify("---"), "");
        let rows = convert_all("Hello World!").unwrap();
        assert_eq!(rows[0].1, "hello-world");
    }
}
