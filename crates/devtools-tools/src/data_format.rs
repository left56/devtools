//! JSON ↔ YAML ↔ TOML converters.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn json_yaml_meta() -> ToolMeta {
    ToolMeta {
        id: ids::JSON_YAML,
        name: "JSON ↔ YAML",
        category: Category::Data,
        summary: "JSON 与 YAML 互转。自动识别输入格式，批量给出两种结果。",
        keywords: &["json", "yaml", "yml", "转换"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn json_toml_meta() -> ToolMeta {
    ToolMeta {
        id: ids::JSON_TOML,
        name: "JSON ↔ TOML",
        category: Category::Data,
        summary: "JSON 与 TOML 互转。自动识别输入格式。",
        keywords: &["json", "toml", "转换"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

fn parse_value(input: &str) -> Result<serde_json::Value, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(t) {
        return Ok(v);
    }
    if let Ok(v) = serde_yaml::from_str::<serde_json::Value>(t) {
        return Ok(v);
    }
    if let Ok(v) = t.parse::<toml::Value>() {
        let json = toml_to_json(v)?;
        return Ok(json);
    }
    Err("无法解析为 JSON / YAML / TOML".into())
}

fn toml_to_json(v: toml::Value) -> Result<serde_json::Value, String> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

fn json_to_toml_value(v: &serde_json::Value) -> Result<toml::Value, String> {
    toml_from_json(v)
}

fn toml_from_json(v: &serde_json::Value) -> Result<toml::Value, String> {
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
                out.push(toml_from_json(item)?);
            }
            Ok(toml::Value::Array(out))
        }
        serde_json::Value::Object(map) => {
            let mut table = toml::map::Map::new();
            for (k, val) in map {
                table.insert(k.clone(), toml_from_json(val)?);
            }
            Ok(toml::Value::Table(table))
        }
    }
}

pub fn json_yaml_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let v = parse_value(input)?;
    let json = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    let yaml = serde_yaml::to_string(&v).map_err(|e| e.to_string())?;
    Ok(vec![("JSON", json), ("YAML", yaml)])
}

pub fn json_toml_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let v = parse_value(input)?;
    let json = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
    let toml_v = json_to_toml_value(&v)?;
    let toml_s = toml::to_string_pretty(&toml_v).map_err(|e| e.to_string())?;
    Ok(vec![("JSON", json), ("TOML", toml_s)])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_to_yaml() {
        let rows = json_yaml_all(r#"{"a":1,"b":true}"#).unwrap();
        assert!(rows[0].1.contains("\"a\""));
        assert!(rows[1].1.contains('a'));
    }
    #[test]
    fn yaml_to_json() {
        let rows = json_yaml_all("a: 1\nb: true\n").unwrap();
        assert!(rows[0].1.contains("\"a\""));
    }
    #[test]
    fn json_to_toml() {
        let rows = json_toml_all(r#"{"name":"x","n":2}"#).unwrap();
        assert!(rows[1].1.contains("name"));
    }
}
