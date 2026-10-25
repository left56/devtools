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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UrlMode {
    Encode,
    Decode,
}

impl UrlMode {
    pub const ALL: [UrlMode; 2] = [UrlMode::Encode, UrlMode::Decode];

    pub fn label(self) -> &'static str {
        match self {
            Self::Encode => "编码",
            Self::Decode => "解码",
        }
    }

    pub fn default_mode() -> Self {
        Self::Encode
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::URL_ENCODE,
        name: "URL Encode",
        category: Category::Encoding,
        summary: "URL 组件编码/解码切换；含 % 时默认解码。",
        keywords: &["url", "encode", "percent", "uri", "编码", "百分号", "解码"],
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

/// Prefer decode when input looks percent-encoded and decodes cleanly.
pub fn suggest_mode(input: &str) -> UrlMode {
    let t = input.trim();
    if t.contains('%')
        && let Ok(decoded) = decode(t)
        && decoded != t
    {
        return UrlMode::Decode;
    }
    UrlMode::Encode
}

pub fn convert_mode(input: &str, mode: UrlMode) -> Result<Vec<(&'static str, String)>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("输入为空".into());
    }
    match mode {
        UrlMode::Encode => Ok(vec![("编码", encode(trimmed))]),
        UrlMode::Decode => {
            let decoded = decode(trimmed).map_err(|e| e.to_string())?;
            Ok(vec![("解码", decoded)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suggest_decode_when_percent() {
        assert_eq!(suggest_mode("a%20b"), UrlMode::Decode);
        assert_eq!(suggest_mode("hello world"), UrlMode::Encode);
    }

    #[test]
    fn convert_mode_one_row() {
        let enc = convert_mode("a b", UrlMode::Encode).unwrap();
        assert_eq!(enc, vec![("编码", "a%20b".into())]);
        let dec = convert_mode("a%20b", UrlMode::Decode).unwrap();
        assert_eq!(dec, vec![("解码", "a b".into())]);
    }

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
        let error = decode("%80").unwrap_err();
        assert!(error.message.contains("UTF-8"));
    }
}
