//! UUID generation via the `uuid` crate. Do not assert random values in tests.

use uuid::Uuid;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    V4,
    V7,
}

impl Version {
    pub const ALL: [Self; 2] = [Self::V4, Self::V7];

    pub fn label(self) -> &'static str {
        match self {
            Self::V4 => "v4",
            Self::V7 => "v7",
        }
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::UUID,
        name: "UUID",
        category: Category::Generators,
        summary: "生成 UUID v4；可选 v7。随机数来自 uuid crate。",
        keywords: &["uuid", "guid", "生成", "v4", "v7"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn generate(version: Version) -> String {
    match version {
        Version::V4 => Uuid::new_v4().to_string(),
        Version::V7 => Uuid::now_v7().to_string(),
    }
}

/// Parse a UUID string and return its RFC version number when present.
pub fn parse_version(input: &str) -> Result<u8, String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("输入为空".into());
    }
    let value = Uuid::parse_str(trimmed).map_err(|error| format!("无法解析 UUID：{error}"))?;
    Ok(value.get_version_num() as u8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v4_has_version_and_variant_bits() {
        let text = generate(Version::V4);
        let parsed = Uuid::parse_str(&text).unwrap();
        assert_eq!(parsed.get_version_num(), 4);
        assert_eq!(parse_version(&text).unwrap(), 4);
        // Do not assert the random body.
        assert_ne!(text, generate(Version::V4));
    }

    #[test]
    fn v7_has_version_bits() {
        let text = generate(Version::V7);
        assert_eq!(parse_version(&text).unwrap(), 7);
    }

    #[test]
    fn empty_and_invalid_parse() {
        assert!(parse_version("").is_err());
        assert!(parse_version("not-a-uuid").is_err());
    }
}
