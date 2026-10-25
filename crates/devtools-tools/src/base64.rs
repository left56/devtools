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

impl Alphabet {
    pub const ALL: [Alphabet; 2] = [Alphabet::Standard, Alphabet::UrlSafe];

    pub fn label(self) -> &'static str {
        match self {
            Self::Standard => "标准",
            Self::UrlSafe => "URL-safe",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Base64Mode {
    Encode,
    Decode,
}

impl Base64Mode {
    pub const ALL: [Base64Mode; 2] = [Base64Mode::Encode, Base64Mode::Decode];

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
        summary: "Base64 编码/解码切换；编码可选手标准或 URL-safe；像 Base64 时默认解码。",
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

pub fn looks_like_base64(s: &str) -> bool {
    let cleaned: String = s.chars().filter(|c| !c.is_whitespace()).collect();
    if cleaned.len() < 4 {
        return false;
    }
    let charset_ok = cleaned.bytes().all(|b| {
        b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'-' | b'_' | b'=')
    });
    charset_ok
        && decode(s, Alphabet::Standard)
            .or_else(|_| decode(s, Alphabet::UrlSafe))
            .is_ok()
}

/// Prefer decode when input looks like Base64 and decodes as UTF-8.
pub fn suggest_mode(input: &str) -> Base64Mode {
    let t = input.trim();
    if looks_like_base64(t) {
        Base64Mode::Decode
    } else {
        Base64Mode::Encode
    }
}

pub fn convert_mode(
    input: &str,
    mode: Base64Mode,
    alphabet: Alphabet,
) -> Result<Vec<(&'static str, String)>, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("输入为空".into());
    }
    match mode {
        Base64Mode::Encode => Ok(vec![
            ("标准编码", encode(trimmed, Alphabet::Standard)),
            ("URL-safe 编码", encode(trimmed, Alphabet::UrlSafe)),
        ]),
        Base64Mode::Decode => {
            // Prefer the selected alphabet; fall back to the other.
            let primary = decode(trimmed, alphabet);
            let fallback = match alphabet {
                Alphabet::Standard => Alphabet::UrlSafe,
                Alphabet::UrlSafe => Alphabet::Standard,
            };
            match primary {
                Ok(t) => Ok(vec![(
                    if alphabet == Alphabet::Standard {
                        "标准解码"
                    } else {
                        "URL-safe 解码"
                    },
                    t,
                )]),
                Err(_) => {
                    let t = decode(trimmed, fallback).map_err(|e| e.to_string())?;
                    Ok(vec![(
                        if fallback == Alphabet::Standard {
                            "标准解码"
                        } else {
                            "URL-safe 解码"
                        },
                        t,
                    )])
                }
            }
        }
    }
}

pub fn decode(input: &str, alphabet: Alphabet) -> Result<String, Base64Error> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(Base64Error {
            message: "输入为空".into(),
        });
    }
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
        let error = decode("/w==", Alphabet::Standard).unwrap_err();
        assert!(error.message.contains("UTF-8"));
    }

    #[test]
    fn convert_mode_encode_and_decode() {
        let rows = convert_mode("hi", Base64Mode::Encode, Alphabet::Standard).unwrap();
        assert!(rows.iter().any(|r| r.0 == "标准编码" && r.1 == "aGk="));
        let enc = encode("hi", Alphabet::Standard);
        assert_eq!(suggest_mode(&enc), Base64Mode::Decode);
        let rows2 = convert_mode(&enc, Base64Mode::Decode, Alphabet::Standard).unwrap();
        assert_eq!(rows2[0].1, "hi");
        assert_eq!(suggest_mode("hi"), Base64Mode::Encode);
    }
}
