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
pub mod http_request;
pub mod dns_lookup;
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
        http_request::meta(),
        dns_lookup::meta(),
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

/// Tools that use the network workbench (`NetworkUse::Explicit`).
pub fn is_network_tool(id: &str) -> bool {
    matches!(id, ids_list::HTTP_REQUEST | ids_list::DNS_LOOKUP)
}

mod ids_list {
    pub use devtools_core::ids::*;
}

#[cfg(test)]
mod tests {
    use super::*;
    use devtools_core::{Availability, NetworkUse, Registry, ids};

    #[test]
    fn builtin_catalog_is_unique_and_ready() {
        let registry = Registry::new(catalog());
        assert_eq!(registry.tools().len(), 16);
        for tool in registry.tools() {
            assert_eq!(tool.availability, Availability::Ready);
            assert!(registry.contains(tool.id));
        }
        assert!(registry.get(ids::JWT).is_some());
        assert!(registry.get(ids::HTTP_REQUEST).is_some());
        assert!(registry.get(ids::DNS_LOOKUP).is_some());
        assert_eq!(
            registry.get(ids::HTTP_REQUEST).unwrap().network,
            NetworkUse::Explicit
        );
        assert_eq!(
            registry.get(ids::DNS_LOOKUP).unwrap().network,
            NetworkUse::Explicit
        );
        assert!(is_inspector_tool(ids::JWT));
        assert!(is_network_tool(ids::HTTP_REQUEST));
        assert!(is_network_tool(ids::DNS_LOOKUP));
        assert!(is_transform_tool(ids::BASE64));
        assert!(!is_transform_tool(ids::JSON));
        assert!(!is_network_tool(ids::URL_PARSE));
    }
}
