//! XML format + XML ↔ JSON (via quick-xml events).

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use quick_xml::Writer;
use serde_json::{Map, Value};
use std::io::Cursor;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::XML_TOOL,
        name: "XML / JSON",
        category: Category::Data,
        summary: "XML 美化/压缩，或与 JSON 互转。模式切换，默认美化。",
        keywords: &["xml", "json", "format", "美化", "压缩"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XmlMode {
    Beautify,
    Minify,
    Json,
}

impl XmlMode {
    pub const ALL: [XmlMode; 3] = [XmlMode::Beautify, XmlMode::Minify, XmlMode::Json];

    pub fn label(self) -> &'static str {
        match self {
            Self::Beautify => "美化",
            Self::Minify => "压缩",
            Self::Json => "↔ JSON",
        }
    }

    pub fn default_mode() -> Self {
        Self::Beautify
    }
}

pub fn convert_mode(input: &str, mode: XmlMode) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    let is_json = t.starts_with('{') || t.starts_with('[');
    match mode {
        XmlMode::Beautify => {
            let xml = if is_json {
                let v: Value = serde_json::from_str(t).map_err(|e| format!("JSON 解析失败：{e}"))?;
                json_to_xml(&v)?
            } else {
                t.to_string()
            };
            Ok(vec![("美化", prettify_xml(&xml)?)])
        }
        XmlMode::Minify => {
            let xml = if is_json {
                let v: Value = serde_json::from_str(t).map_err(|e| format!("JSON 解析失败：{e}"))?;
                json_to_xml(&v)?
            } else {
                t.to_string()
            };
            Ok(vec![("压缩", minify_xml(&xml)?)])
        }
        XmlMode::Json => {
            if is_json {
                let v: Value = serde_json::from_str(t).map_err(|e| format!("JSON 解析失败：{e}"))?;
                let xml = json_to_xml(&v)?;
                Ok(vec![("XML", prettify_xml(&xml)?)])
            } else {
                let json = xml_to_json(t)?;
                Ok(vec![(
                    "JSON",
                    serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?,
                )])
            }
        }
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    convert_mode(input, XmlMode::default_mode())
}

fn rewrite(xml: &str, indent: bool) -> Result<String, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let cursor = Cursor::new(Vec::new());
    let mut writer = if indent {
        Writer::new_with_indent(cursor, b' ', 2)
    } else {
        Writer::new(cursor)
    };
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(e) => writer.write_event(e).map_err(|e| e.to_string())?,
            Err(e) => return Err(format!("XML 解析错误：{e}")),
        }
        buf.clear();
    }
    String::from_utf8(writer.into_inner().into_inner()).map_err(|e| e.to_string())
}

pub fn prettify_xml(xml: &str) -> Result<String, String> {
    rewrite(xml, true)
}
pub fn minify_xml(xml: &str) -> Result<String, String> {
    rewrite(xml, false)
}

pub fn xml_to_json(xml: &str) -> Result<Value, String> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut stack: Vec<(String, Map<String, Value>)> = Vec::new();
    let mut root: Option<Value> = None;
    let mut text_buf = String::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                text_buf.clear();
                stack.push((elem_name(&e)?, attrs_map(&e)));
            }
            Ok(Event::Empty(e)) => {
                let name = elem_name(&e)?;
                let map = attrs_map(&e);
                let node = if map.is_empty() {
                    Value::Null
                } else {
                    Value::Object(map)
                };
                insert_child(&mut stack, &mut root, name, node);
            }
            Ok(Event::Text(t)) => {
                let text = t.unescape().map_err(|e| e.to_string())?.into_owned();
                if !text.trim().is_empty() {
                    text_buf.push_str(&text);
                }
            }
            Ok(Event::CData(t)) => {
                text_buf.push_str(&String::from_utf8_lossy(&t));
            }
            Ok(Event::End(_)) => {
                let (name, mut map) = stack.pop().ok_or_else(|| "XML 标签不匹配".to_string())?;
                let node = if map.is_empty() && !text_buf.is_empty() {
                    Value::String(std::mem::take(&mut text_buf))
                } else {
                    if !text_buf.is_empty() {
                        map.insert("#text".into(), Value::String(std::mem::take(&mut text_buf)));
                    }
                    Value::Object(map)
                };
                text_buf.clear();
                insert_child(&mut stack, &mut root, name, node);
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(e) => return Err(format!("XML 解析错误：{e}")),
        }
        buf.clear();
    }
    root.ok_or_else(|| "空 XML".into())
}

fn attrs_map(e: &BytesStart<'_>) -> Map<String, Value> {
    let mut map = Map::new();
    for a in e.attributes().flatten() {
        let key = format!("@{}", String::from_utf8_lossy(a.key.as_ref()));
        let val = String::from_utf8_lossy(&a.value).into_owned();
        map.insert(key, Value::String(val));
    }
    map
}

fn elem_name(e: &BytesStart<'_>) -> Result<String, String> {
    String::from_utf8(e.name().as_ref().to_vec()).map_err(|e| e.to_string())
}

fn insert_child(
    stack: &mut [(String, Map<String, Value>)],
    root: &mut Option<Value>,
    name: String,
    node: Value,
) {
    if let Some((_, map)) = stack.last_mut() {
        match map.get_mut(&name) {
            Some(Value::Array(arr)) => arr.push(node),
            Some(existing) => {
                let prev = existing.take();
                *existing = Value::Array(vec![prev, node]);
            }
            None => {
                map.insert(name, node);
            }
        }
    } else {
        *root = Some(Value::Object(Map::from_iter([(name, node)])));
    }
}

pub fn json_to_xml(v: &Value) -> Result<String, String> {
    match v {
        Value::Object(map) if map.len() == 1 => {
            let (k, val) = map.iter().next().unwrap();
            Ok(element_to_xml(k, val))
        }
        Value::Object(map) => {
            let mut body = String::new();
            for (k, val) in map {
                body.push_str(&element_to_xml(k, val));
            }
            Ok(format!("<root>{body}</root>"))
        }
        other => Ok(format!("<root>{}</root>", escape_text(&json_leaf(other)))),
    }
}

fn element_to_xml(name: &str, val: &Value) -> String {
    match val {
        Value::Null => format!("<{name}/>"),
        Value::Array(arr) => arr.iter().map(|v| element_to_xml(name, v)).collect(),
        Value::Object(map) => {
            let mut attrs = String::new();
            let mut body = String::new();
            for (k, v) in map {
                if let Some(rest) = k.strip_prefix('@') {
                    attrs.push_str(&format!(" {rest}=\"{}\"", escape_attr(&json_leaf(v))));
                } else if k == "#text" {
                    body.push_str(&escape_text(&json_leaf(v)));
                } else {
                    body.push_str(&element_to_xml(k, v));
                }
            }
            if body.is_empty() {
                format!("<{name}{attrs}/>")
            } else {
                format!("<{name}{attrs}>{body}</{name}>")
            }
        }
        other => format!("<{name}>{}</{name}>", escape_text(&json_leaf(other))),
    }
}

fn json_leaf(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

fn escape_text(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape_text(s).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn xml_to_json_dup_tags() {
        let pretty = convert_mode("<a><b>1</b><b>2</b></a>", XmlMode::Beautify).unwrap();
        assert_eq!(pretty[0].0, "美化");
        let rows = convert_mode("<a><b>1</b><b>2</b></a>", XmlMode::Json).unwrap();
        assert_eq!(rows[0].0, "JSON");
        assert!(rows[0].1.contains('['));
    }
    #[test]
    fn json_input_to_xml() {
        let rows = convert_mode(r#"{"note":{"to":"Tove"}}"#, XmlMode::Json).unwrap();
        assert_eq!(rows[0].0, "XML");
        assert!(rows[0].1.contains("note"));
    }
}
