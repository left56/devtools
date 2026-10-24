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
pub mod basic_auth;
pub mod binary_tool;
pub mod case;
pub mod chmod_calc;
pub mod color_tool;
pub mod cron_tool;
pub mod data_format;
pub mod diff_tool;
pub mod dns_lookup;
pub mod docker_compose;
pub mod email_norm;
pub mod eta_tool;
pub mod hash;
pub mod hmac_tool;
pub mod html_escape;
pub mod http_request;
pub mod http_status;
pub mod iban_tool;
pub mod ipv4_tool;
pub mod ipv6_ula;
pub mod json;
pub mod jwt;
pub mod list_conv;
pub mod lorem;
pub mod mac_gen;
pub mod markdown_html;
pub mod mime_tool;
pub mod nato_alpha;
pub mod number_base;
pub mod numeronym;
pub mod obfuscator;
pub mod otp_tool;
pub mod percentage;
pub mod regex_tool;
pub mod roman;
pub mod safelink;
pub mod slugify;
pub mod sql_format;
pub mod temperature;
pub mod text_stats;
pub mod timestamp;
pub mod token_gen;
pub mod ua_parse;
pub mod ulid_tool;
pub mod url_encode;
pub mod url_parse;
pub mod uuid_tool;
pub mod xml_tool;
pub mod csv_tool;
pub mod json_diff_tool;
pub mod password_strength;
pub mod keycode_info;
pub mod phone_tool;
pub mod ascii_drawer;
pub mod random_port;
pub mod unicode_escape;
pub mod qr_tool;
pub mod wifi_qr;
pub mod meta_tags;
pub mod svg_placeholder;
pub mod cron_presets;

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
        hmac_tool::meta(),
        uuid_tool::meta(),
        ulid_tool::meta(),
        token_gen::meta(),
        otp_tool::meta(),
        timestamp::meta(),
        number_base::meta(),
        case::meta(),
        slugify::meta(),
        lorem::meta(),
        text_stats::meta(),
        list_conv::meta(),
        email_norm::meta(),
        numeronym::meta(),
        obfuscator::meta(),
        nato_alpha::meta(),
        markdown_html::meta(),
        roman::meta(),
        percentage::meta(),
        chmod_calc::meta(),
        binary_tool::meta(),
        color_tool::meta(),
        temperature::meta(),
        eta_tool::meta(),
        basic_auth::meta(),
        data_format::json_yaml_meta(),
        data_format::json_toml_meta(),
        sql_format::meta(),
        ipv4_tool::subnet_meta(),
        ipv4_tool::addr_meta(),
        ipv4_tool::range_meta(),
        ipv6_ula::meta(),
        mac_gen::meta(),
        mime_tool::meta(),
        http_status::meta(),
        ua_parse::meta(),
        safelink::meta(),
        iban_tool::meta(),
        docker_compose::meta(),
        xml_tool::meta(),
        csv_tool::meta(),
        data_format::yaml_toml_meta(),
        json_diff_tool::meta(),
        password_strength::meta(),
        keycode_info::meta(),
        phone_tool::meta(),
        ascii_drawer::meta(),
        random_port::meta(),
        unicode_escape::meta(),
        qr_tool::meta(),
        wifi_qr::meta(),
        meta_tags::meta(),
        svg_placeholder::meta(),
        cron_presets::meta(),
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
            | ids_list::ULID
            | ids_list::TOKEN
            | ids_list::LOREM
            | ids_list::SLUGIFY
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

/// Tools that use the batch workbench (one input → many labeled rows).
pub fn is_batch_tool(id: &str) -> bool {
    matches!(
        id,
        ids_list::TIMESTAMP
            | ids_list::NUMBER_BASE
            | ids_list::TEXT_STATS
            | ids_list::ROMAN
            | ids_list::PERCENTAGE
            | ids_list::CHMOD
            | ids_list::BINARY
            | ids_list::COLOR
            | ids_list::BASIC_AUTH
            | ids_list::HMAC
            | ids_list::OTP
            | ids_list::EMAIL_NORM
            | ids_list::LIST_CONV
            | ids_list::JSON_YAML
            | ids_list::JSON_TOML
            | ids_list::SQL_FORMAT
            | ids_list::MARKDOWN_HTML
            | ids_list::IPV4_SUBNET
            | ids_list::IPV4_ADDR
            | ids_list::IPV4_RANGE
            | ids_list::IPV6_ULA
            | ids_list::MAC_GEN
            | ids_list::MIME
            | ids_list::HTTP_STATUS
            | ids_list::UA_PARSE
            | ids_list::IBAN
            | ids_list::TEMPERATURE
            | ids_list::ETA
            | ids_list::NUMERONYM
            | ids_list::OBFUSCATE
            | ids_list::NATO
            | ids_list::SAFELINK
            | ids_list::DOCKER_RUN
            | ids_list::XML_TOOL
            | ids_list::JSON_CSV
            | ids_list::YAML_TOML
            | ids_list::JSON_DIFF
            | ids_list::PASSWORD_STRENGTH
            | ids_list::KEYCODE
            | ids_list::PHONE
            | ids_list::ASCII_DRAW
            | ids_list::RANDOM_PORT
            | ids_list::UNICODE_ESC
            | ids_list::QR_CODE
            | ids_list::WIFI_QR
            | ids_list::META_TAGS
            | ids_list::SVG_PLACEHOLDER
            | ids_list::CRON_PRESETS
    )
}

/// Batch tools that need an explicit Generate click (random output).
pub fn is_batch_generate_tool(id: &str) -> bool {
    matches!(id, ids_list::IPV6_ULA | ids_list::MAC_GEN | ids_list::RANDOM_PORT)
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
        assert_eq!(registry.tools().len(), 66);
        for tool in registry.tools() {
            assert_eq!(tool.availability, Availability::Ready);
            assert!(registry.contains(tool.id));
        }
        assert!(registry.get(ids::JWT).is_some());
        assert!(registry.get(ids::HMAC).is_some());
        assert!(registry.get(ids::JSON_YAML).is_some());
        assert_eq!(
            registry.get(ids::HTTP_REQUEST).unwrap().network,
            NetworkUse::Explicit
        );
        assert!(is_batch_tool(ids::TIMESTAMP));
        assert!(is_batch_tool(ids::BASIC_AUTH));
        assert!(is_batch_generate_tool(ids::MAC_GEN));
        assert!(is_batch_tool(ids::QR_CODE));
        assert!(is_batch_tool(ids::XML_TOOL));
        assert!(is_batch_generate_tool(ids::RANDOM_PORT));
        assert!(is_transform_tool(ids::TOKEN));
        assert!(!is_transform_tool(ids::TIMESTAMP));
        assert!(!is_network_tool(ids::URL_PARSE));
    }
}
