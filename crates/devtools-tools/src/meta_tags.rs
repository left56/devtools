//! HTML meta / Open Graph snippet generator.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::META_TAGS,
        name: "Meta tags",
        category: Category::Web,
        summary: "生成 HTML meta / Open Graph。输入：标题\\n描述\\nURL\\n可选图片。",
        keywords: &["meta", "og", "open graph", "seo", "标签"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let lines: Vec<_> = input.lines().map(str::trim).collect();
    let title = lines.first().copied().unwrap_or("").trim();
    if title.is_empty() {
        return Err("第 1 行：标题；第 2 行：描述；第 3 行：URL；第 4 行：图片（可选）".into());
    }
    let desc = lines.get(1).copied().unwrap_or("").trim();
    let url = lines.get(2).copied().unwrap_or("").trim();
    let image = lines.get(3).copied().unwrap_or("").trim();
    let mut html = String::new();
    html.push_str(&format!("<title>{}</title>\n", esc(title)));
    if !desc.is_empty() {
        html.push_str(&format!(
            "<meta name=\"description\" content=\"{}\">\n",
            esc(desc)
        ));
    }
    html.push_str(&format!(
        "<meta property=\"og:title\" content=\"{}\">\n",
        esc(title)
    ));
    if !desc.is_empty() {
        html.push_str(&format!(
            "<meta property=\"og:description\" content=\"{}\">\n",
            esc(desc)
        ));
    }
    if !url.is_empty() {
        html.push_str(&format!(
            "<meta property=\"og:url\" content=\"{}\">\n",
            esc(url)
        ));
        html.push_str(&format!("<link rel=\"canonical\" href=\"{}\">\n", esc(url)));
    }
    if !image.is_empty() {
        html.push_str(&format!(
            "<meta property=\"og:image\" content=\"{}\">\n",
            esc(image)
        ));
    }
    html.push_str("<meta property=\"og:type\" content=\"website\">\n");
    html.push_str(&format!(
        "<meta name=\"twitter:card\" content=\"{}\">\n",
        if image.is_empty() {
            "summary"
        } else {
            "summary_large_image"
        }
    ));
    Ok(vec![
        ("HTML", html),
        ("标题", title.to_string()),
        ("描述", if desc.is_empty() { "—".into() } else { desc.into() }),
    ])
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn snippet() {
        let rows = convert_all("Hello\nWorld site\nhttps://ex.com").unwrap();
        assert!(rows[0].1.contains("og:title"));
        assert!(rows[0].1.contains("canonical"));
    }
}
