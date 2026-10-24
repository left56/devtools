//! Markdown → HTML via pulldown-cmark.

use pulldown_cmark::{Options, Parser, html};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::MARKDOWN_HTML,
        name: "Markdown → HTML",
        category: Category::Text,
        summary: "Markdown 转 HTML（CommonMark + tables/strikethrough）。",
        keywords: &["markdown", "html", "md", "转换"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn to_html(input: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);
    let parser = Parser::new_ext(input, options);
    let mut out = String::new();
    html::push_html(&mut out, parser);
    out
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    if input.trim().is_empty() {
        return Err("输入为空".into());
    }
    Ok(vec![("HTML", to_html(input))])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn heading() {
        let html = to_html("# Hi\n\n**bold**");
        assert!(html.contains("<h1>"));
        assert!(html.contains("<strong>bold</strong>"));
    }
}
