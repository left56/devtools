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
        ids::BASIC_AUTH => "BA",
        ids::HMAC => "HM",
        ids::TOKEN => "TK",
        ids::OTP => "otp",
        ids::EMAIL_NORM => "@",
        ids::LIST_CONV => "≡",
        ids::DATA_FORMAT => "{}",
        ids::SQL_FORMAT => "SQL",
        ids::MARKDOWN_HTML => "MD",
        ids::IPV4_SUBNET => "/24",
        ids::IPV4_ADDR => "IP",
        ids::IPV4_RANGE => "…",
        ids::IPV6_ULA => "v6",
        ids::MAC_GEN => "MAC",
        ids::MIME => "MIME",
        ids::HTTP_STATUS => "404",
        ids::UA_PARSE => "UA",
        ids::IBAN => "IB",
        ids::TEMPERATURE => "°",
        ids::ETA => "ETA",
        ids::NUMERONYM => "i18",
        ids::OBFUSCATE => "***",
        ids::NATO => "α",
        ids::SAFELINK => "↗",
        ids::DOCKER_RUN => "dk",
        ids::JSON_CSV => "CSV",
        ids::JSON_DIFF => "Δ",
        ids::PASSWORD_STRENGTH => "PW",
        ids::KEYCODE => "⌨",
        ids::PHONE => "☎",
        ids::ASCII_DRAW => "A",
        ids::RANDOM_PORT => ":P",
        ids::UNICODE_ESC => "U+",
        ids::QR_CODE => "QR",
        ids::WIFI_QR => "WF",
        ids::META_TAGS => "OG",
        ids::SVG_PLACEHOLDER => "▢",
        ids::CRON_PRESETS => "⏱",
        ids::MATH_EVAL => "=",
        ids::GIT_MEMO => "git",
        ids::REGEX_MEMO => "re?",
        ids::BASE64_HEX => "b16",
        ids::BCRYPT => "bc",
        ids::ENCRYPT => "Æ",
        ids::RSA_KEYGEN => "RSA",
        ids::BIP39 => "39",
        ids::XML_TOOL => "<>",
        ids::CERT_DECODE => "CRT",
        ids::BASE64_FILE => "b64",
        ids::IP_LOCATION => "IP",
        ids::NETWORK_INFO => "NIC",
        ids::SPEED_TEST => "⚡",
        ids::WHOIS => "¿?",
        ids::BASE64_IMAGE => "🖼",
        ids::COLOR_PALETTES => "◐",
        ids::RULER => "↕",
        ids::SVG_EDIT => "◇",
        ids::TAILWIND_COLORS => "TW",
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
            Category::Graphics => rgb(0xb89ac8),
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
            Category::Graphics => rgb(0x7c3aed),
        }
    }
}
