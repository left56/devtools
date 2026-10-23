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
pub mod hash;
pub mod html_escape;
pub mod json;
pub mod number_base;
pub mod timestamp;
pub mod url_encode;
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
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use devtools_core::{Availability, NetworkUse, Registry, ids};

    #[test]
    fn builtin_catalog_is_unique_offline_and_ready() {
        let registry = Registry::new(catalog());
        assert_eq!(registry.tools().len(), 9);
        for tool in registry.tools() {
            assert_eq!(tool.network, NetworkUse::Offline);
            assert_eq!(tool.availability, Availability::Ready);
            assert!(registry.contains(tool.id));
        }
        assert!(registry.get(ids::JSON).is_some());
        assert!(registry.get(ids::BASE64).is_some());
        assert!(registry.get(ids::TIMESTAMP).is_some());
        assert!(registry.get(ids::CASE).is_some());
    }
}
