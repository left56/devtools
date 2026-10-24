//! ASCII slugify: lowercase, non-alnum → hyphen, collapse/trim hyphens.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::SLUGIFY,
        name: "Slugify",
        category: Category::Text,
        summary: "把标题变成 URL slug：小写、非字母数字变连字符。",
        keywords: &["slug", "url", "slugify", "kebab", "路径"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
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
    }
}
