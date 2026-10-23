//! Split a URL into components. Never sends a network request.

use std::fmt;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlParseError {
    pub message: String,
}

impl fmt::Display for UrlParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for UrlParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlParts {
    pub href: String,
    pub scheme: String,
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: Option<u16>,
    pub path: String,
    pub query: String,
    pub fragment: String,
    pub query_pairs: Vec<(String, String)>,
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::URL_PARSE,
        name: "URL Parse",
        category: Category::Web,
        summary: "拆分 URL 各段。不发请求，保持离线。",
        keywords: &["url", "parse", "uri", "query", "拆分", "解析"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn parse(input: &str) -> Result<UrlParts, UrlParseError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(UrlParseError {
            message: "输入为空".into(),
        });
    }
    let url = Url::parse(trimmed).map_err(|e| UrlParseError {
        message: format!("无法解析 URL：{e}"),
    })?;

    let query_pairs = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    Ok(UrlParts {
        href: url.to_string(),
        scheme: url.scheme().to_owned(),
        username: url.username().to_owned(),
        password: url.password().unwrap_or("").to_owned(),
        host: url.host_str().unwrap_or("").to_owned(),
        port: url.port_or_known_default(),
        path: url.path().to_owned(),
        query: url.query().unwrap_or("").to_owned(),
        fragment: url.fragment().unwrap_or("").to_owned(),
        query_pairs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_rejected() {
        assert!(parse("").is_err());
    }

    #[test]
    fn typical_https() {
        let p = parse("https://user:pass@example.com:8443/a/b?x=1&y=2#frag").unwrap();
        assert_eq!(p.scheme, "https");
        assert_eq!(p.username, "user");
        assert_eq!(p.password, "pass");
        assert_eq!(p.host, "example.com");
        assert_eq!(p.port, Some(8443));
        assert_eq!(p.path, "/a/b");
        assert_eq!(p.query, "x=1&y=2");
        assert_eq!(p.fragment, "frag");
        assert_eq!(p.query_pairs, vec![("x".into(), "1".into()), ("y".into(), "2".into())]);
    }

    #[test]
    fn invalid_url() {
        assert!(parse("not a url").is_err());
    }

    #[test]
    fn default_https_port() {
        let p = parse("https://example.com/").unwrap();
        assert_eq!(p.port, Some(443));
    }
}
