//! UUID generation via the `uuid` crate (v1, v3–v7).
//!
//! v2 (DCE Security) is not implemented by uuid 1.11 — skipped.
//! Do not assert random UUID bodies in tests.

use uuid::Uuid;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

use crate::quantity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Version {
    V1,
    V3,
    V4,
    V5,
    V6,
    V7,
}

impl Version {
    pub const ALL: [Self; 6] = [
        Self::V1,
        Self::V3,
        Self::V4,
        Self::V5,
        Self::V6,
        Self::V7,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::V1 => "v1",
            Self::V3 => "v3",
            Self::V4 => "v4",
            Self::V5 => "v5",
            Self::V6 => "v6",
            Self::V7 => "v7",
        }
    }

    pub fn needs_name(self) -> bool {
        matches!(self, Self::V3 | Self::V5)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamespaceKind {
    Dns,
    Url,
    Oid,
    X500,
    Custom,
}

impl NamespaceKind {
    pub const ALL: [Self; 5] = [
        Self::Dns,
        Self::Url,
        Self::Oid,
        Self::X500,
        Self::Custom,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Dns => "DNS",
            Self::Url => "URL",
            Self::Oid => "OID",
            Self::X500 => "X500",
            Self::Custom => "自定义",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NameParams {
    pub namespace: NamespaceKind,
    /// Required when `namespace == Custom`.
    pub custom_namespace: String,
    pub name: String,
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::UUID,
        name: "UUID",
        category: Category::Generators,
        summary: "生成 UUID v1/v3/v4/v5/v6/v7（无 v2：uuid crate 未提供）。可设数量。",
        keywords: &["uuid", "guid", "生成", "v1", "v3", "v4", "v5", "v6", "v7"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

fn random_node() -> [u8; 6] {
    let b = *Uuid::new_v4().as_bytes();
    [b[0], b[1], b[2], b[3], b[4], b[5]]
}

fn resolve_namespace(params: &NameParams) -> Result<Uuid, String> {
    match params.namespace {
        NamespaceKind::Dns => Ok(Uuid::NAMESPACE_DNS),
        NamespaceKind::Url => Ok(Uuid::NAMESPACE_URL),
        NamespaceKind::Oid => Ok(Uuid::NAMESPACE_OID),
        NamespaceKind::X500 => Ok(Uuid::NAMESPACE_X500),
        NamespaceKind::Custom => {
            let t = params.custom_namespace.trim();
            if t.is_empty() {
                return Err("自定义命名空间 UUID 不能为空".into());
            }
            Uuid::parse_str(t).map_err(|e| format!("自定义命名空间无效：{e}"))
        }
    }
}

/// Generate one UUID. For v3/v5, `name` must be `Some` with a non-empty name.
pub fn generate(version: Version, name: Option<&NameParams>) -> Result<String, String> {
    let uuid = match version {
        Version::V1 => Uuid::now_v1(&random_node()),
        Version::V4 => Uuid::new_v4(),
        Version::V6 => Uuid::now_v6(&random_node()),
        Version::V7 => Uuid::now_v7(),
        Version::V3 | Version::V5 => {
            let params = name.ok_or_else(|| "v3/v5 需要命名空间与名称".to_string())?;
            let trimmed = params.name.trim();
            if trimmed.is_empty() {
                return Err("名称不能为空（v3/v5 基于名称）".into());
            }
            let ns = resolve_namespace(params)?;
            if version == Version::V3 {
                Uuid::new_v3(&ns, trimmed.as_bytes())
            } else {
                Uuid::new_v5(&ns, trimmed.as_bytes())
            }
        }
    };
    Ok(uuid.to_string())
}

/// Generate `qty` UUIDs (clamped 1..=100), one per line.
pub fn generate_many(
    version: Version,
    qty: usize,
    name: Option<&NameParams>,
) -> Result<String, String> {
    let n = quantity::clamp(qty);
    let mut lines = Vec::with_capacity(n);
    for _ in 0..n {
        lines.push(generate(version, name)?);
    }
    Ok(lines.join("\n"))
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
        let text = generate(Version::V4, None).unwrap();
        let parsed = Uuid::parse_str(&text).unwrap();
        assert_eq!(parsed.get_version_num(), 4);
        assert_eq!(parse_version(&text).unwrap(), 4);
        assert_ne!(text, generate(Version::V4, None).unwrap());
    }

    #[test]
    fn v7_has_version_bits() {
        let text = generate(Version::V7, None).unwrap();
        assert_eq!(parse_version(&text).unwrap(), 7);
    }

    #[test]
    fn v1_and_v6_version_bits() {
        assert_eq!(
            parse_version(&generate(Version::V1, None).unwrap()).unwrap(),
            1
        );
        assert_eq!(
            parse_version(&generate(Version::V6, None).unwrap()).unwrap(),
            6
        );
    }

    #[test]
    fn v5_dns_known_and_v3_version() {
        let params = NameParams {
            namespace: NamespaceKind::Dns,
            custom_namespace: String::new(),
            name: "rust-lang.org".into(),
        };
        let v5 = generate(Version::V5, Some(&params)).unwrap();
        assert_eq!(parse_version(&v5).unwrap(), 5);
        // Stable: same name → same UUID
        assert_eq!(v5, generate(Version::V5, Some(&params)).unwrap());
        let v3 = generate(Version::V3, Some(&params)).unwrap();
        assert_eq!(parse_version(&v3).unwrap(), 3);
    }

    #[test]
    fn v3_without_name_errors() {
        assert!(generate(Version::V3, None).is_err());
        let empty = NameParams {
            namespace: NamespaceKind::Dns,
            custom_namespace: String::new(),
            name: "  ".into(),
        };
        assert!(generate(Version::V3, Some(&empty)).is_err());
    }

    #[test]
    fn generate_many_line_count() {
        let text = generate_many(Version::V4, 5, None).unwrap();
        assert_eq!(text.lines().count(), 5);
        let capped = generate_many(Version::V4, 500, None).unwrap();
        assert_eq!(capped.lines().count(), quantity::MAX);
    }

    #[test]
    fn empty_and_invalid_parse() {
        assert!(parse_version("").is_err());
        assert!(parse_version("not-a-uuid").is_err());
    }
}
