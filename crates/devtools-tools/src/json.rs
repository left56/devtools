use std::fmt;

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use serde_json::Value;

mod tree;
pub use tree::{JsonDocument, JsonKind, JsonNode};

/// JSON parse failure. Positions match `serde_json`: 1-based line and byte
/// column; the column can be zero for empty input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "第 {} 行、第 {} 列：{}",
            self.line, self.column, self.message
        )
    }
}

impl std::error::Error for JsonError {}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::JSON,
        name: "JSON",
        category: Category::Data,
        summary: "严格格式化、压缩和树形检查 JSON。保留键顺序，显示节点路径和 jq 表达式。",
        keywords: &[
            "json",
            "format",
            "pretty",
            "minify",
            "格式化",
            "压缩",
            "tree",
            "path",
            "jq",
            "树",
            "路径",
        ],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Pretty-print strict JSON with two-space indentation.
///
/// Object key order follows the input. `serde_json` only does that when the
/// `preserve_order` feature is on; the workspace dependency enables it.
pub fn prettify(input: &str) -> Result<String, JsonError> {
    Ok(serde_json::to_string_pretty(&parse(input)?)
        .expect("serializing a just-parsed Value cannot fail"))
}

/// Emit compact strict JSON. Key order follows the input.
pub fn minify(input: &str) -> Result<String, JsonError> {
    Ok(serde_json::to_string(&parse(input)?).expect("serializing a just-parsed Value cannot fail"))
}

fn parse(input: &str) -> Result<Value, JsonError> {
    serde_json::from_str(input).map_err(|error| JsonError {
        message: error.to_string(),
        line: error.line(),
        column: error.column(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prettify_keeps_key_order_and_minify_round_trips() {
        let input = r#"{"b":1,"a":2}"#;
        assert_eq!(prettify(input).unwrap(), "{\n  \"b\": 1,\n  \"a\": 2\n}");
        assert_eq!(minify(input).unwrap(), input);
        assert_eq!(minify(&prettify(input).unwrap()).unwrap(), input);
    }

    #[test]
    fn unicode_strings_round_trip() {
        let input = r#"{"name":"你好"}"#;
        assert_eq!(minify(input).unwrap(), input);
    }

    #[test]
    fn trailing_comma_is_rejected() {
        let error = prettify(r#"{"a":1,}"#).unwrap_err();
        assert!(error.line >= 1);
        assert!(error.column >= 1);
    }

    #[test]
    fn empty_input_is_rejected() {
        assert!(minify("").is_err());
        assert!(minify("   ").is_err());
    }
}
