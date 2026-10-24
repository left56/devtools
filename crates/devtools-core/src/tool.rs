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
