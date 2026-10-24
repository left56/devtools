//! TOTP (RFC 6238) generate / validate. Secret is Base32.

use std::time::{SystemTime, UNIX_EPOCH};

use hmac::{Hmac, Mac};
use sha1::Sha1;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

type HmacSha1 = Hmac<Sha1>;

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::OTP,
        name: "OTP / TOTP",
        category: Category::Security,
        summary: "TOTP 生成与校验（HMAC-SHA1，30 秒）。输入 Base32 密钥；可选第二行验证码。",
        keywords: &["otp", "totp", "2fa", "mfa", "authenticator"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

fn decode_base32(input: &str) -> Result<Vec<u8>, String> {
    let cleaned: String = input
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '=')
        .map(|c| c.to_ascii_uppercase())
        .collect();
    if cleaned.is_empty() {
        return Err("Base32 密钥为空".into());
    }
    let mut buffer: u32 = 0;
    let mut bits: u32 = 0;
    let mut out = Vec::new();
    for ch in cleaned.bytes() {
        let val = ALPHABET
            .iter()
            .position(|&c| c == ch)
            .ok_or_else(|| format!("非法 Base32 字符 `{}`", ch as char))? as u32;
        buffer = (buffer << 5) | val;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Ok(out)
}

fn hotp(key: &[u8], counter: u64, digits: u32) -> Result<String, String> {
    let mut mac = HmacSha1::new_from_slice(key).map_err(|_| "密钥无效".to_string())?;
    mac.update(&counter.to_be_bytes());
    let hash = mac.finalize().into_bytes();
    let offset = (hash[19] & 0x0f) as usize;
    let bin = ((u32::from(hash[offset]) & 0x7f) << 24)
        | (u32::from(hash[offset + 1]) << 16)
        | (u32::from(hash[offset + 2]) << 8)
        | u32::from(hash[offset + 3]);
    let modulo = 10u32.pow(digits);
    Ok(format!("{:0digits$}", bin % modulo, digits = digits as usize))
}

fn now_step(period: u64) -> u64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    secs / period
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let mut lines = input.lines().map(str::trim).filter(|l| !l.is_empty());
    let secret = lines.next().ok_or_else(|| "请输入 Base32 密钥".to_string())?;
    let maybe_code = lines.next();
    let key = decode_base32(secret)?;
    let period = 30u64;
    let digits = 6u32;
    let step = now_step(period);
    let current = hotp(&key, step, digits)?;
    let prev = hotp(&key, step.saturating_sub(1), digits)?;
    let next = hotp(&key, step + 1, digits)?;
    let remaining = period - (SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
        % period);
    let mut rows = vec![
        ("当前 TOTP", current.clone()),
        ("上一个", prev.clone()),
        ("下一个", next.clone()),
        ("剩余秒数", remaining.to_string()),
        ("周期", format!("{period}s")),
    ];
    if let Some(code) = maybe_code {
        let ok = code == current || code == prev || code == next;
        rows.push(("校验", if ok { "通过（含 ±1 窗口）".into() } else { "不匹配".into() }));
    }
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rfc6238_sha1_vectors() {
        // secret "12345678901234567890" as ASCII, not base32 — use known base32 for "Hello!"
        // Direct hotp with ASCII key from RFC:
        let key = b"12345678901234567890";
        assert_eq!(hotp(key, 1, 8).unwrap(), "94287082");
        assert_eq!(hotp(key, 0x00000000023523EC, 8).unwrap(), "07081804");
    }

    #[test]
    fn base32_and_rows() {
        // "Hello!" -> JBSWY3DPEBLW64TMMQQQ====
        let rows = convert_all("JBSWY3DPEBLW64TMMQQQ").unwrap();
        assert_eq!(rows[0].0, "当前 TOTP");
        assert_eq!(rows[0].1.len(), 6);
    }
}
