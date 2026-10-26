//! Digest helpers. MD5 and SHA-1 are for checksums and compatibility only.

use base64::Engine;
use md5::Md5;
use ripemd::Ripemd160;
use sha1::Sha1;
use sha2::{Digest, Sha224, Sha256, Sha384, Sha512};
use sha3::{Keccak256, Sha3_256, Sha3_512};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Sha3_512,
    Keccak256,
    Ripemd160,
}

impl Algorithm {
    pub const ALL: [Self; 10] = [
        Self::Md5,
        Self::Sha1,
        Self::Sha224,
        Self::Sha256,
        Self::Sha384,
        Self::Sha512,
        Self::Sha3_256,
        Self::Sha3_512,
        Self::Keccak256,
        Self::Ripemd160,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA-1",
            Self::Sha224 => "SHA-224",
            Self::Sha256 => "SHA-256",
            Self::Sha384 => "SHA-384",
            Self::Sha512 => "SHA-512",
            Self::Sha3_256 => "SHA3-256",
            Self::Sha3_512 => "SHA3-512",
            Self::Keccak256 => "Keccak-256",
            Self::Ripemd160 => "RIPEMD-160",
        }
    }

    pub fn row_label(self) -> &'static str {
        match self {
            Self::Md5 => "MD5（不用于密码）",
            Self::Sha1 => "SHA-1（不用于密码）",
            other => other.label(),
        }
    }

    /// Shown next to MD5 / SHA-1 in the shell.
    pub fn safety_note(self) -> Option<&'static str> {
        match self {
            Self::Md5 | Self::Sha1 => Some("不用于密码或完整性安全"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashEncoding {
    Hex,
    HexUpper,
    Base64,
    Bin,
}

impl HashEncoding {
    pub const ALL: [Self; 4] = [Self::Hex, Self::HexUpper, Self::Base64, Self::Bin];

    pub fn label(self) -> &'static str {
        match self {
            Self::Hex => "Hex",
            Self::HexUpper => "HEX",
            Self::Base64 => "Base64",
            Self::Bin => "Bin",
        }
    }

    pub fn default_encoding() -> Self {
        Self::Hex
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::HASH,
        name: "Hasher",
        category: Category::Security,
        summary: "Hasher：MD5/SHA/SHA3/Keccak/RIPEMD。可切换 Hex/HEX/Base64/Bin。弱算法已标注。",
        keywords: &[
            "hash", "hasher", "md5", "sha1", "sha224", "sha256", "sha384", "sha512", "sha3",
            "keccak", "ripemd", "摘要", "校验",
        ],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Digests of the UTF-8 bytes of `input`. Empty input is allowed.
pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    convert_with_encoding(input, HashEncoding::Hex)
}

pub fn convert_with_encoding(
    input: &str,
    encoding: HashEncoding,
) -> Result<Vec<(&'static str, String)>, String> {
    let mut rows: Vec<(&'static str, String)> = Algorithm::ALL
        .into_iter()
        .map(|algo| (algo.row_label(), format_digest(input, algo, encoding)))
        .collect();
    rows.push(("字节数", input.len().to_string()));
    rows.push(("编码", encoding.label().to_string()));
    Ok(rows)
}

pub fn digest(input: &str, algorithm: Algorithm) -> String {
    format_digest(input, algorithm, HashEncoding::Hex)
}

fn digest_bytes(input: &str, algorithm: Algorithm) -> Vec<u8> {
    let bytes = input.as_bytes();
    match algorithm {
        Algorithm::Md5 => Md5::digest(bytes).to_vec(),
        Algorithm::Sha1 => Sha1::digest(bytes).to_vec(),
        Algorithm::Sha224 => Sha224::digest(bytes).to_vec(),
        Algorithm::Sha256 => Sha256::digest(bytes).to_vec(),
        Algorithm::Sha384 => Sha384::digest(bytes).to_vec(),
        Algorithm::Sha512 => Sha512::digest(bytes).to_vec(),
        Algorithm::Sha3_256 => Sha3_256::digest(bytes).to_vec(),
        Algorithm::Sha3_512 => Sha3_512::digest(bytes).to_vec(),
        Algorithm::Keccak256 => Keccak256::digest(bytes).to_vec(),
        Algorithm::Ripemd160 => Ripemd160::digest(bytes).to_vec(),
    }
}

fn format_digest(input: &str, algorithm: Algorithm, encoding: HashEncoding) -> String {
    let raw = digest_bytes(input, algorithm);
    match encoding {
        HashEncoding::Hex => hex(&raw),
        HashEncoding::HexUpper => hex(&raw).to_ascii_uppercase(),
        HashEncoding::Base64 => base64::engine::general_purpose::STANDARD.encode(&raw),
        HashEncoding::Bin => raw
            .iter()
            .map(|b| format!("{b:08b}"))
            .collect::<Vec<_>>()
            .join(" "),
    }
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string_has_known_digests() {
        assert_eq!(digest("", Algorithm::Md5), "d41d8cd98f00b204e9800998ecf8427e");
        assert_eq!(
            digest("", Algorithm::Sha256),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn typical_ascii() {
        assert_eq!(digest("abc", Algorithm::Md5), "900150983cd24fb0d6963f7d28e17f72");
        assert_eq!(
            digest("abc", Algorithm::Sha1),
            "a9993e364706816aba3e25717850c26c9cd0d89d"
        );
        assert_eq!(
            digest("abc", Algorithm::Sha224),
            "23097d223405d8228642a477bda255b32aadbce4bda0b3f7e36c9da7"
        );
        assert_eq!(
            digest("abc", Algorithm::Sha256),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            digest("abc", Algorithm::Sha512),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
        assert_eq!(
            digest("abc", Algorithm::Sha3_256),
            "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
        );
        assert_eq!(
            digest("abc", Algorithm::Ripemd160),
            "8eb208f7e05d987a9b044a8e98c6b087f15a0bfc"
        );
    }

    #[test]
    fn encoding_base64_and_bin() {
        let hex = digest("abc", Algorithm::Md5);
        let rows = convert_with_encoding("abc", HashEncoding::Base64).unwrap();
        let b64 = rows.iter().find(|r| r.0.starts_with("MD5")).unwrap().1.clone();
        assert_ne!(b64, hex);
        assert!(b64.ends_with('=') || b64.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/'));
        let rows_bin = convert_with_encoding("a", HashEncoding::Bin).unwrap();
        let md5_bin = rows_bin.iter().find(|r| r.0.starts_with("MD5")).unwrap().1.clone();
        assert!(md5_bin.contains('0') && md5_bin.contains('1') && md5_bin.contains(' '));
    }

    #[test]
    fn unicode_is_hashed_as_utf8_bytes() {
        let value = digest("你好", Algorithm::Sha256);
        assert_eq!(value.len(), 64);
        assert!(value.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn safety_notes_only_on_weak_algos() {
        assert!(Algorithm::Md5.safety_note().is_some());
        assert!(Algorithm::Sha1.safety_note().is_some());
        assert!(Algorithm::Sha256.safety_note().is_none());
    }

    #[test]
    fn convert_all_lists_family() {
        let rows = convert_all("abc").unwrap();
        assert!(rows.len() >= 8);
        assert_eq!(
            rows.iter().find(|r| r.0.starts_with("MD5")).unwrap().1,
            "900150983cd24fb0d6963f7d28e17f72"
        );
    }
}
