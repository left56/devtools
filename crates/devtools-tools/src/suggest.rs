//! Conservative clipboard guesses.
//!
//! False negatives are acceptable. A guess never switches the active tool.
//! Rules for tools that are not registered yet still run here, and
//! [`applicable_suggestions`] drops them so the shell cannot offer a tool
//! that has no implementation.

use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use devtools_core::{Confidence, Detection, Registry, ids};
use serde_json::Value;

/// Zero or one guess. The `Vec` is the stable shape so a later secondary
/// hint does not change the caller.
pub fn suggest(input: &str) -> Vec<Detection> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let detection = if looks_like_jwt(trimmed) {
        Detection {
            tool_id: ids::JWT,
            confidence: Confidence::High,
            reason: "三段式 JWT。只识别结构，不校验签名。",
        }
    } else if looks_like_json_document(trimmed) {
        Detection {
            tool_id: ids::JSON,
            confidence: Confidence::High,
            reason: "可以按严格 JSON 解析的对象或数组。",
        }
    } else if looks_like_unix_timestamp(trimmed) {
        Detection {
            tool_id: ids::TIMESTAMP,
            confidence: Confidence::Medium,
            reason: "10 位秒或 13 位毫秒，落在 2001–2096 年。",
        }
    } else if looks_like_standard_base64(trimmed) {
        Detection {
            tool_id: ids::BASE64,
            confidence: Confidence::Medium,
            reason: "标准 Base64（不是 URL-safe），解码后是文本。",
        }
    } else {
        return Vec::new();
    };

    vec![detection]
}

/// Suggestions the shell may show: the guess exists and the tool is registered.
pub fn applicable_suggestions(registry: &Registry, input: &str) -> Vec<Detection> {
    suggest(input)
        .into_iter()
        .filter(|detection| registry.contains(detection.tool_id))
        .collect()
}

fn looks_like_json_document(input: &str) -> bool {
    if !(input.starts_with('{') || input.starts_with('[')) {
        return false;
    }
    serde_json::from_str::<Value>(input).is_ok()
}

fn looks_like_unix_timestamp(input: &str) -> bool {
    if !matches!(input.len(), 10 | 13) || !input.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    let Ok(value) = input.parse::<u64>() else {
        return false;
    };
    let seconds = if input.len() == 13 {
        value / 1000
    } else {
        value
    };
    (1_000_000_000..4_000_000_000).contains(&seconds)
}

fn looks_like_jwt(input: &str) -> bool {
    let mut parts = input.split('.');
    let Some(header) = parts.next() else {
        return false;
    };
    let Some(payload) = parts.next() else {
        return false;
    };
    let Some(signature) = parts.next() else {
        return false;
    };
    if parts.next().is_some() || header.is_empty() || payload.is_empty() {
        return false;
    }

    let Some(header_json) = decode_json_segment(header) else {
        return false;
    };
    let Some(payload_json) = decode_json_segment(payload) else {
        return false;
    };
    if !payload_json.is_object() {
        return false;
    }
    let Some(algorithm) = header_json.get("alg").and_then(Value::as_str) else {
        return false;
    };
    if signature.is_empty() {
        algorithm == "none"
    } else {
        is_base64url(signature)
    }
}

fn decode_json_segment(segment: &str) -> Option<Value> {
    if !is_base64url(segment) {
        return None;
    }
    let bytes = URL_SAFE_NO_PAD.decode(segment).ok()?;
    serde_json::from_slice(&bytes).ok()
}

fn is_base64url(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn looks_like_standard_base64(input: &str) -> bool {
    if input.len() < 16 || !input.len().is_multiple_of(4) || !has_base64_signal(input) {
        return false;
    }
    if !input
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
    {
        return false;
    }
    let Ok(bytes) = STANDARD.decode(input) else {
        return false;
    };
    if bytes.len() < 8 || STANDARD.encode(&bytes) != input {
        return false;
    }
    let Ok(text) = std::str::from_utf8(&bytes) else {
        return false;
    };
    let total = text.chars().count();
    if total == 0 {
        return false;
    }
    let readable = text
        .chars()
        .filter(|character| character.is_whitespace() || !character.is_control())
        .count();
    readable * 4 >= total * 3
}

/// Digits or padding distinguish Base64 from a long alphabetic word.
/// Missing those is a false negative we accept.
fn has_base64_signal(input: &str) -> bool {
    input
        .bytes()
        .any(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'/' | b'='))
}

#[cfg(test)]
mod tests {
    use super::*;
    use devtools_core::ids;

    fn jwt(header: &str, payload: &str, signature: &str) -> String {
        format!(
            "{}.{}.{}",
            URL_SAFE_NO_PAD.encode(header),
            URL_SAFE_NO_PAD.encode(payload),
            signature
        )
    }

    #[test]
    fn json_object_and_array_are_high_confidence() {
        let object = suggest(r#"  {"b":1,"a":2}  "#);
        assert_eq!(object[0].tool_id, ids::JSON);
        assert_eq!(object[0].confidence, Confidence::High);
        assert_eq!(suggest("[1, 2, 3]")[0].tool_id, ids::JSON);
    }

    #[test]
    fn json_scalars_and_broken_json_are_not_guessed() {
        assert!(suggest(r#""just a string""#).is_empty());
        assert!(suggest(r#"{"a":1,}"#).is_empty());
        assert!(suggest("").is_empty());
    }

    #[test]
    fn timestamp_is_medium_and_not_offered_until_registered() {
        let guess = suggest("1700000000");
        assert_eq!(guess[0].tool_id, ids::TIMESTAMP);
        assert_eq!(guess[0].confidence, Confidence::Medium);
        assert_eq!(suggest("1700000000000")[0].tool_id, ids::TIMESTAMP);
        assert!(suggest("0000000000").is_empty());

        let registry = Registry::new(crate::catalog());
        assert!(applicable_suggestions(&registry, "1700000000").is_empty());
        assert_eq!(
            applicable_suggestions(&registry, r#"{"ok":true}"#)[0].tool_id,
            ids::JSON
        );
    }

    #[test]
    fn jwt_structure_is_detected_without_calling_it_valid() {
        let token = jwt(
            r#"{"alg":"HS256","typ":"JWT"}"#,
            r#"{"sub":"1"}"#,
            "signature",
        );
        let guess = suggest(&token);
        assert_eq!(guess[0].tool_id, ids::JWT);
        assert!(!guess[0].reason.contains("有效"));
        assert!(suggest(&jwt(r#"{"alg":"none"}"#, r#"{"sub":"1"}"#, "")).len() == 1);
        assert!(suggest(&jwt(r#"{"typ":"JWT"}"#, r#"{"sub":"1"}"#, "signature")).is_empty());
    }

    #[test]
    fn base64_requires_a_non_alphabetic_signal_and_canonical_padding() {
        let encoded = STANDARD.encode("hello world");
        assert_eq!(encoded, "aGVsbG8gd29ybGQ=");
        assert_eq!(suggest(&encoded)[0].tool_id, ids::BASE64);
        assert!(suggest("abcdefghijklmnop").is_empty());
        assert!(suggest("aGk=").is_empty());
        assert!(suggest(encoded.trim_end_matches('=')).is_empty());
    }
}
