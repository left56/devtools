//! JWT decode only. Never treats a decoded token as verified or trusted.

use std::fmt;

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use chrono::{TimeZone, Utc};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JwtError {
    pub message: String,
}

impl fmt::Display for JwtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for JwtError {}

/// Decode result. There is intentionally no `verified` / `trusted` field.
#[derive(Debug, Clone, PartialEq)]
pub struct JwtDecode {
    pub header_raw: String,
    pub payload_raw: String,
    pub signature_raw: String,
    pub header: Value,
    pub payload: Value,
    /// Exact `alg` string from the header, when present.
    pub alg: Option<String>,
    pub alg_is_none: bool,
    /// Parsed `exp` as UTC display + whether it is in the past, when present.
    pub exp: Option<ExpInfo>,
    /// Always true for this tool — decoding is not verification.
    pub unsigned_notice: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpInfo {
    pub unix_seconds: i64,
    pub utc_display: String,
    pub expired: bool,
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::JWT,
        name: "JWT",
        category: Category::Security,
        summary: "只解码 JWT。显示 header/payload/签名原文与 alg；不验签。",
        keywords: &["jwt", "token", "decode", "解码", "未验签"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn decode(input: &str) -> Result<JwtDecode, JwtError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(JwtError {
            message: "输入为空".into(),
        });
    }
    let mut parts = trimmed.split('.');
    let header_b64 = parts.next().unwrap_or("");
    let payload_b64 = parts.next().unwrap_or("");
    let signature_b64 = parts.next().unwrap_or("");
    if parts.next().is_some() {
        return Err(JwtError {
            message: "JWT 应恰好有两或三个以 `.` 分隔的段".into(),
        });
    }
    if header_b64.is_empty() || payload_b64.is_empty() {
        return Err(JwtError {
            message: "缺少 header 或 payload 段".into(),
        });
    }

    let header = decode_json_part(header_b64, "header")?;
    let payload = decode_json_part(payload_b64, "payload")?;
    let header_raw = serde_json::to_string_pretty(&header).unwrap_or_else(|_| header.to_string());
    let payload_raw =
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| payload.to_string());

    let alg = header
        .get("alg")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let alg_is_none = alg
        .as_deref()
        .is_some_and(|a| a.eq_ignore_ascii_case("none"));

    let exp = payload
        .get("exp")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_f64().map(|f| f as i64))
                .or_else(|| v.as_u64().map(|u| u as i64))
        })
        .and_then(|seconds| {
            let dt = Utc.timestamp_opt(seconds, 0).single()?;
            Some(ExpInfo {
                unix_seconds: seconds,
                utc_display: dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                expired: dt < Utc::now(),
            })
        });

    Ok(JwtDecode {
        header_raw,
        payload_raw,
        signature_raw: signature_b64.to_owned(),
        header,
        payload,
        alg,
        alg_is_none,
        exp,
        unsigned_notice: "未验签 · 本工具只解码，不验证签名或信任 token",
    })
}

fn decode_json_part(segment: &str, name: &str) -> Result<Value, JwtError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(segment)
        .or_else(|_| {
            // Some tokens include padding.
            base64::engine::general_purpose::URL_SAFE.decode(segment)
        })
        .map_err(|e| JwtError {
            message: format!("{name} 段 Base64URL 解码失败：{e}"),
        })?;
    serde_json::from_slice(&bytes).map_err(|e| JwtError {
        message: format!("{name} 段不是 JSON：{e}"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn token(header: &str, payload: &str, sig: &str) -> String {
        format!(
            "{}.{}.{}",
            URL_SAFE_NO_PAD.encode(header),
            URL_SAFE_NO_PAD.encode(payload),
            sig
        )
    }

    #[test]
    fn empty_is_rejected() {
        assert!(decode("").is_err());
    }

    #[test]
    fn typical_hs256_decodes_without_trust() {
        let t = token(
            r#"{"alg":"HS256","typ":"JWT"}"#,
            r#"{"sub":"1","name":"test"}"#,
            "signature",
        );
        let out = decode(&t).unwrap();
        assert_eq!(out.alg.as_deref(), Some("HS256"));
        assert!(!out.alg_is_none);
        assert_eq!(out.signature_raw, "signature");
        assert!(out.unsigned_notice.contains("未验签"));
        // No success/trusted field exists on the type — compile-time guarantee.
        let _ = &out.header;
    }

    #[test]
    fn alg_none_is_flagged_untrusted() {
        let t = token(r#"{"alg":"none","typ":"JWT"}"#, r#"{"sub":"1"}"#, "");
        let out = decode(&t).unwrap();
        assert!(out.alg_is_none);
        assert_eq!(out.alg.as_deref(), Some("none"));
        assert!(out.unsigned_notice.contains("未验签"));
    }

    #[test]
    fn expired_exp_is_reported_without_claiming_valid() {
        let t = token(
            r#"{"alg":"HS256"}"#,
            r#"{"exp":1000000000}"#, // 2001-09-09, long expired
            "sig",
        );
        let out = decode(&t).unwrap();
        let exp = out.exp.expect("exp");
        assert!(exp.expired);
        assert!(exp.utc_display.contains("UTC"));
        assert!(out.unsigned_notice.contains("未验签"));
    }

    #[test]
    fn broken_segment_errors() {
        assert!(decode("a.b").is_err());
        assert!(decode("not-jwt").is_err());
    }
}
