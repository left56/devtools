//! Per-tool glyphs for the sidebar and palette.
//!
//! Shape-distinct ASCII/Unicode marks — no SVG assets. Category tints are
//! muted (Refactoring UI) and only reinforce grouping; icons stay readable
//! in monochrome.

use crate::theme::Theme;
use devtools_core::{Category, ids};
use gpui::{Rgba, rgb};

/// Short glyph shown beside a tool name. Always shape-distinct across tools.
pub fn tool_glyph(tool_id: &str) -> &'static str {
    match tool_id {
        ids::JSON => "{}",
        ids::BASE64 => "64",
        ids::URL_ENCODE => "%",
        ids::HTML_ESCAPE => "<>",
        ids::HASH => "#",
        ids::UUID => "◈",
        ids::TIMESTAMP => "◷",
        ids::NUMBER_BASE => "0x",
        ids::CASE => "Aa",
        ids::JWT => "◉",
        ids::REGEX => ".*",
        ids::CRON => "⟳",
        ids::URL_PARSE => "◎",
        ids::DIFF => "±",
        ids::HTTP_REQUEST => "↗",
        ids::DNS_LOOKUP => "⌖",
        ids::ULID => "▴",
        ids::SLUGIFY => "––",
        ids::LOREM => "¶",
        ids::TEXT_STATS => "∑",
        ids::ROMAN => "Ⅻ",
        ids::PERCENTAGE => "%",
        ids::CHMOD => "rwx",
        ids::BINARY => "01",
        ids::COLOR => "▣",
        _ => "·",
    }
}

/// Muted category tint for the glyph (not for the tool name).
pub fn category_tint(category: Category, theme: &Theme) -> Rgba {
    if theme.is_dark {
        match category {
            Category::Data => rgb(0x7a9cc8),
            Category::Encoding => rgb(0x6aa896),
            Category::Text => rgb(0xb89a6e),
            Category::Time => rgb(0x8490a8),
            Category::Generators => rgb(0x6fbf8f),
            Category::Web => rgb(0x6b93e8),
            Category::Security => rgb(0xc49090),
        }
    } else {
        match category {
            Category::Data => rgb(0x3568d4),
            Category::Encoding => rgb(0x0f766e),
            Category::Text => rgb(0x9a6b10),
            Category::Time => rgb(0x475569),
            Category::Generators => rgb(0x157a3c),
            Category::Web => rgb(0x2563eb),
            Category::Security => rgb(0xb42318),
        }
    }
}
