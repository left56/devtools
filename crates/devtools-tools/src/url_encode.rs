//! URL component encode/decode (not full URL parsing).

use std::fmt;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};

/// Characters percent-encoded by `encodeURIComponent` in browsers, plus the
/// extra set that form-urlencoded historically escapes. We follow the common
/// "component" set: encode everything except unreserved URI characters.
const COMPONENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'$')
    .add(b'%')
    .add(b'&')
    .add(b'+')
    .add(b',')
    .add(b'/')
    .add(b':')
    .add(b';')
    .add(b'<')
    .add(b'=')
    .add(b'>')
    .add(b'?')
    .add(b'@')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'`')
    .add(b'{')
    .add(b'|')
    .add(b'}');

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UrlEncodeError {
    pub message: String,
}

impl fmt::Display for UrlEncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for UrlEncodeError {}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::URL_ENCODE,
        name: "URL Encode",
        category: Category::Encoding,
        summary: "URL 组件编解码，不是整段 URL 解析。",
        keywords: &["url", "encode", "percent", "uri", "编码", "百分号"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn encode(input: &str) -> String {
    utf8_percent_encode(input, COMPONENT).to_string()
}

pub fn decode(input: &str) -> Result<String, UrlEncodeError> {
    if input.is_empty() {
        return Err(UrlEncodeError {
            message: "输入为空".into(),
        });
    }
    percent_decode_str(input)
        .decode_utf8()
        .map(|cow| cow.into_owned())
        .map_err(|_| UrlEncodeError {
            message: "百分号解码结果不是合法 UTF-8".into(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_decode_is_rejected() {
        assert!(decode("").is_err());
    }

    #[test]
    fn encodes_reserved_and_round_trips() {
        let original = "a b/中文?x=1&y=2";
        let encoded = encode(original);
        assert!(encoded.contains("%20"));
        assert!(encoded.contains("%2F") || encoded.contains("%2f"));
        assert!(!encoded.contains(' '));
        assert_eq!(decode(&encoded).unwrap(), original);
    }

    #[test]
    fn typical_query_value() {
        assert_eq!(encode("hello world"), "hello%20world");
        assert_eq!(decode("hello%20world").unwrap(), "hello world");
    }

    #[test]
    fn invalid_percent_sequence_is_rejected() {
        // Lone % leaves invalid UTF-8 when treated as raw bytes with 0x25 followed by junk;
        // percent-encoding crate may keep %GG as literal bytes. Force a bad UTF-8 path:
        let error = decode("%80").unwrap_err();
        assert!(error.message.contains("UTF-8"));
    }
}
