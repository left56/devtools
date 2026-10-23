//! Standard and URL-safe Base64 encode/decode. Default alphabet is standard.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE, URL_SAFE_NO_PAD};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alphabet {
    Standard,
    UrlSafe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Base64Error {
    pub message: String,
}

impl fmt::Display for Base64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Base64Error {}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::BASE64,
        name: "Base64",
        category: Category::Encoding,
        summary: "标准 Base64 编解码。URL-safe 可作为开关，默认标准。",
        keywords: &["base64", "编码", "解码", "url-safe"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn encode(input: &str, alphabet: Alphabet) -> String {
    match alphabet {
        Alphabet::Standard => STANDARD.encode(input.as_bytes()),
        Alphabet::UrlSafe => URL_SAFE.encode(input.as_bytes()),
    }
}

pub fn decode(input: &str, alphabet: Alphabet) -> Result<String, Base64Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Base64Error {
            message: "输入为空".into(),
        });
    }
    // Accept both padded and unpadded URL-safe. Standard requires padding when needed.
    let bytes = match alphabet {
        Alphabet::Standard => STANDARD.decode(trimmed).map_err(|error| Base64Error {
            message: format!("标准 Base64 解码失败：{error}"),
        })?,
        Alphabet::UrlSafe => URL_SAFE
            .decode(trimmed)
            .or_else(|_| URL_SAFE_NO_PAD.decode(trimmed))
            .map_err(|error| Base64Error {
                message: format!("URL-safe Base64 解码失败：{error}"),
            })?,
    };
    String::from_utf8(bytes).map_err(|_| Base64Error {
        message: "解码结果不是合法 UTF-8 文本".into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_decode_is_rejected() {
        assert!(decode("", Alphabet::Standard).is_err());
        assert!(decode("   ", Alphabet::UrlSafe).is_err());
    }

    #[test]
    fn standard_round_trip() {
        let original = "hello world /+=";
        let encoded = encode(original, Alphabet::Standard);
        assert_eq!(encoded, "aGVsbG8gd29ybGQgLys9");
        assert_eq!(decode(&encoded, Alphabet::Standard).unwrap(), original);
    }

    #[test]
    fn url_safe_avoids_plus_slash() {
        let original = "subjects?_d";
        let encoded = encode(original, Alphabet::UrlSafe);
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));
        assert_eq!(decode(&encoded, Alphabet::UrlSafe).unwrap(), original);
    }

    #[test]
    fn invalid_standard_is_rejected() {
        let error = decode("!!!!", Alphabet::Standard).unwrap_err();
        assert!(error.message.contains("解码失败"));
    }

    #[test]
    fn non_utf8_payload_is_rejected() {
        // 0xff alone is invalid UTF-8; /w== is standard Base64 for [0xff].
        let error = decode("/w==", Alphabet::Standard).unwrap_err();
        assert!(error.message.contains("UTF-8"));
    }
}
