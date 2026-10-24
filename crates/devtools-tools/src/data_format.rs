//! Unified JSON / YAML / TOML / XML converter.
//!
//! Auto-detects input (overridable). Emits one chosen output format
//! (default = first in [`FormatKind::ALL`]).

use crate::xml_tool;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatKind {
    Json,
    Yaml,
    Toml,
    Xml,
}

impl FormatKind {
    pub const ALL: [FormatKind; 4] = [
        FormatKind::Json,
        FormatKind::Yaml,
        FormatKind::Toml,
        FormatKind::Xml,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
            Self::Xml => "XML",
        }
    }

    pub fn default_output() -> Self {
        Self::ALL[0]
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::DATA_FORMAT,
        name: "数据格式",
        category: Category::Data,
        summary: "JSON / YAML / TOML / XML 互转：自动识别输入，可改输入与输出类型。",
        keywords: &[
            "json", "yaml", "yml", "toml", "xml", "转换", "data", "format", "数据格式",
        ],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Best-effort detect. Prefers JSON, then YAML, TOML, XML.
pub fn detect(input: &str) -> Option<FormatKind> {
    let t = input.trim();
    if t.is_empty() {
        return None;
    }
    if serde_json::from_str::<serde_json::Value>(t).is_ok() {
        return Some(FormatKind::Json);
    }
    // XML before YAML: YAML can accept almost anything as a string scalar.
    if looks_like_xml(t) && xml_tool::xml_to_json(t).is_ok() {
        return Some(FormatKind::Xml);
    }
    if t.parse::<toml::Value>().is_ok() && looks_like_toml(t) {
        return Some(FormatKind::Toml);
    }
    if serde_yaml::from_str::<serde_json::Value>(t).is_ok() {
        return Some(FormatKind::Yaml);
    }
    None
}

fn looks_like_xml(t: &str) -> bool {
    let s = t.trim_start();
    s.starts_with('<') || s.starts_with("<?xml")
}

fn looks_like_toml(t: &str) -> bool {
    // Avoid claiming plain YAML `key: value` as TOML when it has no `=`.
    t.contains('=') || t.trim_start().starts_with('[')
}

pub fn parse_as(input: &str, kind: FormatKind) -> Result<serde_json::Value, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    match kind {
        FormatKind::Json => {
            serde_json::from_str(t).map_err(|e| format!("JSON 解析失败：{e}"))
        }
        FormatKind::Yaml => {
            serde_yaml::from_str(t).map_err(|e| format!("YAML 解析失败：{e}"))
        }
        FormatKind::Toml => {
            let v = t
                .parse::<toml::Value>()
                .map_err(|e| format!("TOML 解析失败：{e}"))?;
            serde_json::to_value(v).map_err(|e| e.to_string())
        }
        FormatKind::Xml => xml_tool::xml_to_json(t),
    }
}

pub fn emit(value: &serde_json::Value, kind: FormatKind) -> Result<String, String> {
    match kind {
        FormatKind::Json => serde_json::to_string_pretty(value).map_err(|e| e.to_string()),
        FormatKind::Yaml => serde_yaml::to_string(value).map_err(|e| e.to_string()),
        FormatKind::Toml => {
            let toml_v = json_to_toml_value(value)?;
            toml::to_string_pretty(&toml_v).map_err(|e| e.to_string())
        }
        FormatKind::Xml => {
            let xml = xml_tool::json_to_xml(value)?;
            xml_tool::prettify_xml(&xml)
        }
    }
}

/// `input_kind = None` means auto-detect.
pub fn convert(
    input: &str,
    input_kind: Option<FormatKind>,
    output_kind: FormatKind,
) -> Result<(FormatKind, String), String> {
    let resolved = match input_kind {
        Some(k) => k,
        None => detect(input).ok_or_else(|| "无法识别为 JSON / YAML / TOML / XML".to_string())?,
    };
    let value = parse_as(input, resolved)?;
    let text = emit(&value, output_kind)?;
    Ok((resolved, text))
}

/// BatchWorkbench row helper: one labeled result for the chosen output.
pub fn convert_row(
    input: &str,
    input_kind: Option<FormatKind>,
    output_kind: FormatKind,
) -> Result<Vec<(&'static str, String)>, String> {
    let (_detected, text) = convert(input, input_kind, output_kind)?;
    Ok(vec![(output_kind.label(), text)])
}

fn json_to_toml_value(v: &serde_json::Value) -> Result<toml::Value, String> {
    match v {
        serde_json::Value::Null => Err("TOML 不支持 null".into()),
        serde_json::Value::Bool(b) => Ok(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(toml::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                toml::Value::try_from(f).map_err(|e| e.to_string())
            } else {
                Err("数字无法写入 TOML".into())
            }
        }
        serde_json::Value::String(s) => Ok(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let mut out = Vec::new();
            for item in arr {
                out.push(json_to_toml_value(item)?);
            }
            Ok(toml::Value::Array(out))
        }
        serde_json::Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (k, val) in map {
                table.insert(k.clone(), json_to_toml_value(val)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}

/// Remap retired sidebar twins onto the unified tool.
pub fn remap_legacy_id(id: &str) -> &str {
    match id {
        ids::JSON_YAML | ids::JSON_TOML | ids::YAML_TOML | ids::XML_TOOL => ids::DATA_FORMAT,
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_json_yaml_toml_xml() {
        assert_eq!(detect(r#"{"a":1}"#), Some(FormatKind::Json));
        assert_eq!(detect("a: 1\nb: true\n"), Some(FormatKind::Yaml));
        assert_eq!(detect("a = 1\nb = true\n"), Some(FormatKind::Toml));
        assert_eq!(detect("<root><a>1</a></root>"), Some(FormatKind::Xml));
    }

    #[test]
    fn json_to_yaml_row() {
        let rows = convert_row(r#"{"a":1,"b":true}"#, None, FormatKind::Yaml).unwrap();
        assert_eq!(rows[0].0, "YAML");
        assert!(rows[0].1.contains('a'));
    }

    #[test]
    fn override_input_yaml_as_json_fails() {
        assert!(convert("a: 1\n", Some(FormatKind::Json), FormatKind::Json).is_err());
    }

    #[test]
    fn xml_round_via_json() {
        let (_k, xml) = convert(
            r#"{"note":{"to":"Tove"}}"#,
            Some(FormatKind::Json),
            FormatKind::Xml,
        )
        .unwrap();
        assert!(xml.contains("note"));
        let (_k, json) = convert(&xml, Some(FormatKind::Xml), FormatKind::Json).unwrap();
        assert!(json.contains("note"));
    }

    #[test]
    fn default_output_is_first() {
        assert_eq!(FormatKind::default_output(), FormatKind::Json);
        assert_eq!(FormatKind::ALL[0], FormatKind::Json);
    }

    #[test]
    fn remap_legacy() {
        assert_eq!(remap_legacy_id(ids::JSON_YAML), ids::DATA_FORMAT);
        assert_eq!(remap_legacy_id(ids::XML_TOOL), ids::DATA_FORMAT);
        assert_eq!(remap_legacy_id(ids::JSON), ids::JSON);
    }
}
