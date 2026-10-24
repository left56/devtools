//! Digest helpers. MD5 and SHA-1 are for checksums and compatibility only.

use md5::Md5;
use sha1::{Digest, Sha1};
use sha2::{Sha256, Sha384, Sha512};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Algorithm {
    Md5,
    Sha1,
    Sha256,
    Sha384,
    Sha512,
}

impl Algorithm {
    pub const ALL: [Self; 5] = [Self::Md5, Self::Sha1, Self::Sha256, Self::Sha384, Self::Sha512];

    pub fn label(self) -> &'static str {
        match self {
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA-1",
            Self::Sha256 => "SHA-256",
            Self::Sha384 => "SHA-384",
            Self::Sha512 => "SHA-512",
        }
    }

    /// Shown next to MD5 / SHA-1 in the shell.
    pub fn safety_note(self) -> Option<&'static str> {
        match self {
            Self::Md5 | Self::Sha1 => Some("不用于密码或完整性安全"),
            Self::Sha256 | Self::Sha384 | Self::Sha512 => None,
        }
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::HASH,
        name: "Hash",
        category: Category::Security,
        summary: "一次输入给出 MD5/SHA-1/SHA-256/SHA-384/SHA-512。弱算法已标注。",
        keywords: &["hash", "md5", "sha1", "sha256", "sha384", "sha512", "摘要", "校验"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Hex digest of the UTF-8 bytes of `input`. Empty input is allowed (empty-string digest).
pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    // Empty allowed — empty-string digests are well-defined.
    let mut rows: Vec<(&'static str, String)> = Algorithm::ALL
        .into_iter()
        .map(|algo| {
            let label = match algo {
                Algorithm::Md5 => "MD5（不用于密码）",
                Algorithm::Sha1 => "SHA-1（不用于密码）",
                Algorithm::Sha256 => "SHA-256",
                Algorithm::Sha384 => "SHA-384",
                Algorithm::Sha512 => "SHA-512",
            };
            (label, digest(input, algo))
        })
        .collect();
    rows.push(("字节数", input.len().to_string()));
    Ok(rows)
}

pub fn digest(input: &str, algorithm: Algorithm) -> String {
    let bytes = input.as_bytes();
    match algorithm {
        Algorithm::Md5 => hex(Md5::digest(bytes)),
        Algorithm::Sha1 => hex(Sha1::digest(bytes)),
        Algorithm::Sha256 => hex(Sha256::digest(bytes)),
        Algorithm::Sha384 => hex(Sha384::digest(bytes)),
        Algorithm::Sha512 => hex(Sha512::digest(bytes)),
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
            digest("abc", Algorithm::Sha256),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            digest("abc", Algorithm::Sha512),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
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
        assert!(rows.len() >= 5);
        assert_eq!(
            rows.iter().find(|r| r.0.starts_with("MD5")).unwrap().1,
            "900150983cd24fb0d6963f7d28e17f72"
        );
    }
}
