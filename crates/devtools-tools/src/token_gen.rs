//! Random token generator (explicit generate).

use uuid::Uuid;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alphabet {
    Hex,
    Alphanumeric,
    Base64Url,
    Digits,
}

impl Alphabet {
    pub const ALL: [Self; 4] = [Self::Hex, Self::Alphanumeric, Self::Base64Url, Self::Digits];
    pub fn label(self) -> &'static str {
        match self {
            Self::Hex => "hex",
            Self::Alphanumeric => "字母数字",
            Self::Base64Url => "base64url",
            Self::Digits => "数字",
        }
    }
    fn charset(self) -> &'static [u8] {
        match self {
            Self::Hex => b"0123456789abcdef",
            Self::Alphanumeric => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
            Self::Base64Url => b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_",
            Self::Digits => b"0123456789",
        }
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::TOKEN,
        name: "Token",
        category: Category::Generators,
        summary: "生成随机令牌。可选长度与字母表；点击生成。",
        keywords: &["token", "random", "secret", "令牌", "随机"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn generate(length: usize, alphabet: Alphabet) -> String {
    let len = length.clamp(1, 512);
    let set = alphabet.charset();
    let mut out = String::with_capacity(len);
    // Draw entropy from UUID v4 bytes repeatedly.
    while out.len() < len {
        for b in Uuid::new_v4().as_bytes() {
            if out.len() >= len {
                break;
            }
            out.push(set[(*b as usize) % set.len()] as char);
        }
    }
    out
}

pub fn parse_length(input: &str, default: usize) -> usize {
    let t = input.trim();
    if t.is_empty() {
        return default;
    }
    t.parse::<usize>().unwrap_or(default).clamp(1, 512)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn length_and_charset() {
        let t = generate(32, Alphabet::Hex);
        assert_eq!(t.len(), 32);
        assert!(t.bytes().all(|b| b.is_ascii_hexdigit()));
        assert_ne!(t, generate(32, Alphabet::Hex));
    }
}
