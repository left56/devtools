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
pub mod quantity;
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
pub mod math_eval;
pub mod git_memo;
pub mod regex_memo;
pub mod base64_hex;
pub mod bcrypt_tool;
pub mod encrypt_tool;
pub mod rsa_tool;
pub mod bip39_tool;
pub mod cert_decode;
pub mod base64_file;
pub mod ip_location;
pub mod network_info;
pub mod speed_test;
pub mod whois_tool;
pub mod page_fetch;
pub mod base64_image;
pub mod color_palettes;
pub mod ruler_tool;
pub mod svg_edit;
pub mod tailwind_colors;

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
        data_format::meta(),
        xml_tool::meta(),
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
        csv_tool::meta(),
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
        math_eval::meta(),
        git_memo::meta(),
        regex_memo::meta(),
        base64_hex::meta(),
        bcrypt_tool::meta(),
        encrypt_tool::meta(),
        rsa_tool::meta(),
        bip39_tool::meta(),
        cert_decode::meta(),
        base64_file::meta(),
        ip_location::meta(),
        network_info::meta(),
        whois_tool::meta(),
        page_fetch::meta(),
        base64_image::meta(),
        color_palettes::meta(),
        ruler_tool::meta(),
        svg_edit::meta(),
        tailwind_colors::meta(),
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
    // Generators only. Converters use BatchWorkbench (one input → all outputs).
    matches!(
        id,
        ids_list::UUID | ids_list::ULID | ids_list::TOKEN | ids_list::LOREM
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
            | ids_list::BASE64
            | ids_list::URL_ENCODE
            | ids_list::HTML_ESCAPE
            | ids_list::HASH
            | ids_list::CASE
            | ids_list::SLUGIFY
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
            | ids_list::DATA_FORMAT
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
            | ids_list::JSON_CSV
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
            | ids_list::MATH_EVAL
            | ids_list::GIT_MEMO
            | ids_list::REGEX_MEMO
            | ids_list::BASE64_HEX
            | ids_list::BCRYPT
            | ids_list::ENCRYPT
            | ids_list::RSA_KEYGEN
            | ids_list::BIP39
            | ids_list::XML_TOOL
            | ids_list::CERT_DECODE
            | ids_list::BASE64_FILE
            | ids_list::BASE64_IMAGE
            | ids_list::COLOR_PALETTES
            | ids_list::RULER
            | ids_list::SVG_EDIT
            | ids_list::TAILWIND_COLORS
    )
}

/// Batch converters whose input is a single value (CIDR, status code, …).
///
/// These must use a single-line field in `BatchWorkbench`, not a tall textarea.
/// Tools that accept documents, lists, payloads, or multi-field line protocols
/// stay multiline.
pub fn is_batch_single_line_tool(id: &str) -> bool {
    matches!(
        id,
        ids_list::TIMESTAMP
            | ids_list::CASE
            | ids_list::SLUGIFY
            | ids_list::NUMBER_BASE
            | ids_list::ROMAN
            | ids_list::PERCENTAGE
            | ids_list::CHMOD
            | ids_list::COLOR
            | ids_list::COLOR_PALETTES
            | ids_list::RULER
            | ids_list::TAILWIND_COLORS
            | ids_list::BASIC_AUTH
            | ids_list::EMAIL_NORM
            | ids_list::IPV4_SUBNET
            | ids_list::IPV4_ADDR
            | ids_list::IPV4_RANGE
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
            | ids_list::PASSWORD_STRENGTH
            | ids_list::KEYCODE
            | ids_list::PHONE
            | ids_list::MATH_EVAL
            | ids_list::CRON_PRESETS
            | ids_list::GIT_MEMO
            | ids_list::REGEX_MEMO
    )
}

/// Batch tools that need an explicit Generate click (random output).
pub fn is_batch_generate_tool(id: &str) -> bool {
    matches!(id, ids_list::IPV6_ULA | ids_list::MAC_GEN | ids_list::RANDOM_PORT | ids_list::RSA_KEYGEN | ids_list::BIP39)
}

/// Tools that use the network workbench (`NetworkUse::Explicit`).
pub fn is_network_tool(id: &str) -> bool {
    matches!(
        id,
        ids_list::HTTP_REQUEST
            | ids_list::DNS_LOOKUP
            | ids_list::IP_LOCATION
            | ids_list::NETWORK_INFO
            | ids_list::WHOIS
            | ids_list::PAGE_FETCH
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
    fn builtin_catalog_is_unique_and_ready() {
        let registry = Registry::new(catalog());
        assert_eq!(registry.tools().len(), 83);
        for tool in registry.tools() {
            assert_eq!(tool.availability, Availability::Ready);
            assert!(registry.contains(tool.id));
        }
        assert!(registry.get(ids::JWT).is_some());
        assert!(registry.get(ids::HMAC).is_some());
        assert!(registry.get(ids::DATA_FORMAT).is_some());
        assert!(registry.get(ids::JSON_YAML).is_none());
        assert_eq!(
            registry.get(ids::HTTP_REQUEST).unwrap().network,
            NetworkUse::Explicit
        );
        assert!(is_batch_tool(ids::TIMESTAMP));
        assert!(is_batch_tool(ids::BASE64));
        assert!(is_batch_tool(ids::HASH));
        assert!(is_batch_tool(ids::CASE));
        assert!(!is_transform_tool(ids::BASE64));
        assert!(!is_transform_tool(ids::HASH));
        assert!(is_batch_tool(ids::BASIC_AUTH));
        assert!(is_batch_generate_tool(ids::MAC_GEN));
        assert!(is_batch_tool(ids::QR_CODE));
        assert!(is_batch_tool(ids::DATA_FORMAT));
        assert!(is_batch_tool(ids::XML_TOOL));
        assert!(is_batch_tool(ids::CERT_DECODE));
        assert!(is_batch_tool(ids::BASE64_FILE));
        assert!(is_batch_tool(ids::BASE64_IMAGE));
        assert!(is_batch_tool(ids::TAILWIND_COLORS));
        assert!(is_network_tool(ids::IP_LOCATION));
        assert!(is_network_tool(ids::WHOIS));
        assert!(is_network_tool(ids::NETWORK_INFO));
        assert!(!is_network_tool(ids::SPEED_TEST));
        assert!(is_network_tool(ids::PAGE_FETCH));
        assert!(is_batch_tool(ids::MATH_EVAL));
        assert!(is_batch_generate_tool(ids::RSA_KEYGEN));
        assert!(is_batch_generate_tool(ids::RANDOM_PORT));
        assert!(is_transform_tool(ids::TOKEN));
        assert!(!is_transform_tool(ids::TIMESTAMP));
        assert!(!is_network_tool(ids::URL_PARSE));
        assert!(is_batch_single_line_tool(ids::IPV4_SUBNET));
        assert!(is_batch_single_line_tool(ids::IPV4_ADDR));
        assert!(is_batch_single_line_tool(ids::HTTP_STATUS));
        assert!(is_batch_single_line_tool(ids::TIMESTAMP));
        assert!(!is_batch_single_line_tool(ids::BASE64));
        assert!(!is_batch_single_line_tool(ids::HMAC));
        assert!(!is_batch_single_line_tool(ids::LIST_CONV));
        assert!(!is_batch_single_line_tool(ids::DATA_FORMAT));
        assert!(!is_batch_single_line_tool(ids::WIFI_QR));
    }
}
