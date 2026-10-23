//! Pure tool behavior. No GPUI types.
//!
//! A function in this crate is the authority for what a tool does. Clipboard
//! guesses in [`suggest`] are deliberately lossy and must not be treated as
//! the tool's result.

#![forbid(unsafe_code)]

pub mod json;

mod suggest;

use devtools_core::ToolMeta;

pub use suggest::{applicable_suggestions, suggest};

pub fn catalog() -> Vec<ToolMeta> {
    vec![json::meta()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use devtools_core::{Registry, ids};

    #[test]
    fn builtin_catalog_is_unique_and_json_is_offline() {
        let registry = Registry::new(catalog());
        let json = registry.get(ids::JSON).expect("json is registered");
        assert_eq!(json.network, devtools_core::NetworkUse::Offline);
        assert_eq!(json.availability, devtools_core::Availability::Ready);
        assert_eq!(registry.tools().len(), 1);
    }
}
