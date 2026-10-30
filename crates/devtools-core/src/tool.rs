use crate::Category;

/// Stable tool ids. Display names may change; these strings may not.
///
/// An id existing here does not mean the tool is registered. The registry is
/// the list of tools the shell is allowed to offer.
pub mod ids {
    pub const JSON: &str = "json";
    pub const BASE64: &str = "base64";
    pub const URL_ENCODE: &str = "url-encode";
    pub const HTML_ESCAPE: &str = "html-escape";
    pub const HASH: &str = "hash";
    pub const UUID: &str = "uuid";
    pub const TIMESTAMP: &str = "timestamp";
    pub const NUMBER_BASE: &str = "number-base";
    pub const CASE: &str = "case";
    pub const JWT: &str = "jwt";
    pub const REGEX: &str = "regex";
    pub const CRON: &str = "cron";
    pub const URL_PARSE: &str = "url-parse";
    pub const DIFF: &str = "diff";
    pub const HTTP_REQUEST: &str = "http-request";
    pub const DNS_LOOKUP: &str = "dns-lookup";
    pub const ULID: &str = "ulid";
    pub const SLUGIFY: &str = "slugify";
    pub const LOREM: &str = "lorem";
    pub const TEXT_STATS: &str = "text-stats";
    pub const ROMAN: &str = "roman";
    pub const PERCENTAGE: &str = "percentage";
    pub const CHMOD: &str = "chmod";
    pub const BINARY: &str = "binary";
    pub const COLOR: &str = "color";
    pub const BASIC_AUTH: &str = "basic-auth";
    pub const HMAC: &str = "hmac";
    pub const TOKEN: &str = "token";
    pub const OTP: &str = "otp";
    pub const EMAIL_NORM: &str = "email-norm";
    pub const LIST_CONV: &str = "list-conv";
    pub const DATA_FORMAT: &str = "data-format";
    /// Legacy ids kept for prefs remap → [`DATA_FORMAT`].
    pub const JSON_YAML: &str = "json-yaml";
    pub const JSON_TOML: &str = "json-toml";
    pub const SQL_FORMAT: &str = "sql-format";
    pub const MARKDOWN_HTML: &str = "markdown-html";
    pub const IPV4_SUBNET: &str = "ipv4-subnet";
    pub const IPV4_ADDR: &str = "ipv4-addr";
    pub const IPV4_RANGE: &str = "ipv4-range";
    pub const IPV6_ULA: &str = "ipv6-ula";
    pub const MAC_GEN: &str = "mac-gen";
    pub const MIME: &str = "mime";
    pub const HTTP_STATUS: &str = "http-status";
    pub const UA_PARSE: &str = "ua-parse";
    pub const IBAN: &str = "iban";
    pub const TEMPERATURE: &str = "temperature";
    pub const ETA: &str = "eta";
    pub const NUMERONYM: &str = "numeronym";
    pub const OBFUSCATE: &str = "obfuscate";
    pub const NATO: &str = "nato";
    pub const SAFELINK: &str = "safelink";
    pub const DOCKER_RUN: &str = "docker-run";
    pub const XML_TOOL: &str = "xml-tool";
    pub const JSON_CSV: &str = "json-csv";
    pub const YAML_TOML: &str = "yaml-toml";
    pub const JSON_DIFF: &str = "json-diff";
    pub const PASSWORD_STRENGTH: &str = "password-strength";
    pub const KEYCODE: &str = "keycode";
    pub const PHONE: &str = "phone";
    pub const ASCII_DRAW: &str = "ascii-draw";
    pub const RANDOM_PORT: &str = "random-port";
    pub const UNICODE_ESC: &str = "unicode-esc";
    pub const QR_CODE: &str = "qr-code";
    pub const WIFI_QR: &str = "wifi-qr";
    pub const META_TAGS: &str = "meta-tags";
    pub const SVG_PLACEHOLDER: &str = "svg-placeholder";
    pub const CRON_PRESETS: &str = "cron-presets";
    pub const MATH_EVAL: &str = "math-eval";
    pub const GIT_MEMO: &str = "git-memo";
    pub const REGEX_MEMO: &str = "regex-memo";
    pub const BASE64_HEX: &str = "base64-hex";
    pub const BCRYPT: &str = "bcrypt";
    pub const ENCRYPT: &str = "encrypt";
    pub const RSA_KEYGEN: &str = "rsa-keygen";
    pub const BIP39: &str = "bip39";
    pub const CERT_DECODE: &str = "cert-decode";
    pub const BASE64_FILE: &str = "base64-file";
    pub const IP_LOCATION: &str = "ip-location";
    pub const NETWORK_INFO: &str = "network-info";
    pub const SPEED_TEST: &str = "speed-test";
    pub const WHOIS: &str = "whois";
    pub const BASE64_IMAGE: &str = "base64-image";
    pub const COLOR_PALETTES: &str = "color-palettes";
    pub const RULER: &str = "ruler";
    pub const SVG_EDIT: &str = "svg-edit";
    pub const TAILWIND_COLORS: &str = "tailwind-colors";
    pub const PAGE_FETCH: &str = "page-fetch";
    pub const LOG_VIEWER: &str = "log-viewer";
}


/// Whether running the tool contacts the network.
///
/// The shell must show [`NetworkUse::Explicit`] before the tool runs.
/// Defaulting a new tool to offline is the point of the type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkUse {
    Offline,
    Explicit,
}

/// Whether the shell can drive the tool yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Availability {
    /// The Rust function exists and is tested. The workspace editor does not
    /// call it yet.
    LogicOnly,
    Ready,
}

/// How distinctive a clipboard guess is.
///
/// High means the structure is hard to confuse with ordinary text. The shell
/// still must not switch tools on its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Medium,
    High,
}

/// A guess that `input` might belong to `tool_id`.
///
/// This is not a validation result. A JWT detection does not mean the
/// signature was checked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Detection {
    pub tool_id: &'static str,
    pub confidence: Confidence,
    pub reason: &'static str,
}

/// Catalog entry. Pure data, no behavior.
///
/// Interactive tools (regex highlights, HTTP, a JSON tree) are not modeled as
/// `fn(&str) -> String`. Those tools keep their state in the shell and call
/// plain functions in `devtools-tools`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolMeta {
    pub id: &'static str,
    pub name: &'static str,
    pub category: Category,
    pub summary: &'static str,
    pub keywords: &'static [&'static str],
    pub network: NetworkUse,
    pub availability: Availability,
}
