//! Pure tool behavior. No GPUI types.
//!
//! A function in this crate is the authority for what a tool does. Clipboard
//! guesses in [`suggest`] are deliberately lossy and must not be treated as
//! the tool's result.
//!
//! All modules here must stay portable (macOS / Linux / Windows). Do not call
//! platform UI or path APIs from tool logic.

#![forbid(unsafe_code)]

pub mod base64;
pub mod case;
pub mod cron_tool;
pub mod diff_tool;
pub mod hash;
pub mod html_escape;
pub mod json;
pub mod jwt;
pub mod number_base;
pub mod regex_tool;
pub mod timestamp;
pub mod url_encode;
pub mod url_parse;
pub mod uuid_tool;

mod suggest;

use devtools_core::ToolMeta;

pub use suggest::{applicable_suggestions, suggest};

pub fn catalog() -> Vec<ToolMeta> {
    vec![
        json::meta(),
        base64::meta(),
        url_encode::meta(),
        html_escape::meta(),
        hash::meta(),
        uuid_tool::meta(),
        timestamp::meta(),
        number_base::meta(),
        case::meta(),
        jwt::meta(),
        regex_tool::meta(),
        cron_tool::meta(),
        url_parse::meta(),
        diff_tool::meta(),
    ]
}

/// Tools that use the shared transform workbench (single in / out).
pub fn is_transform_tool(id: &str) -> bool {
    matches!(
        id,
        ids_list::BASE64
            | ids_list::URL_ENCODE
            | ids_list::HTML_ESCAPE
            | ids_list::HASH
            | ids_list::UUID
            | ids_list::TIMESTAMP
            | ids_list::NUMBER_BASE
            | ids_list::CASE
    )
}

/// Tools that use the inspector workbench (structured results).
pub fn is_inspector_tool(id: &str) -> bool {
    matches!(
        id,
        ids_list::JWT
            | ids_list::REGEX
            | ids_list::CRON
            | ids_list::URL_PARSE
            | ids_list::DIFF
    )
}

mod ids_list {
    pub use devtools_core::ids::*;
}

#[cfg(test)]
mod tests {
    use super::*;
    use devtools_core::{Availability, NetworkUse, Registry, ids};

    #[test]
    fn builtin_catalog_is_unique_offline_and_ready() {
        let registry = Registry::new(catalog());
        assert_eq!(registry.tools().len(), 14);
        for tool in registry.tools() {
            assert_eq!(tool.network, NetworkUse::Offline);
            assert_eq!(tool.availability, Availability::Ready);
            assert!(registry.contains(tool.id));
        }
        assert!(registry.get(ids::JWT).is_some());
        assert!(registry.get(ids::REGEX).is_some());
        assert!(registry.get(ids::CRON).is_some());
        assert!(registry.get(ids::URL_PARSE).is_some());
        assert!(registry.get(ids::DIFF).is_some());
        assert!(is_inspector_tool(ids::JWT));
        assert!(is_transform_tool(ids::BASE64));
        assert!(!is_transform_tool(ids::JSON));
    }
}
