//! A strict JSON document and stable, ordered inspection nodes. No UI types.
use super::{JsonError, parse};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonKind {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
}

impl JsonKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Object => "对象",
            Self::Array => "数组",
            Self::String => "字符串",
            Self::Number => "数字",
            Self::Boolean => "布尔值",
            Self::Null => "null",
        }
    }
    pub fn is_container(self) -> bool {
        matches!(self, Self::Object | Self::Array)
    }
}

#[derive(Debug)]
pub struct JsonNode {
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub label: String,
    pub kind: JsonKind,
    pub preview: String,
    pub json_path: String,
    /// A jq filter, not a shell command. Use without wrapping it in shell quotes.
    pub jq: String,
    pointer: String,
}

#[derive(Debug)]
pub struct JsonDocument {
    value: Value,
    nodes: Vec<JsonNode>,
}

fn quoted(value: &str) -> String {
    serde_json::to_string(value).expect("serializing a string")
}

impl JsonDocument {
    pub fn parse(input: &str) -> Result<Self, JsonError> {
        let value = parse(input)?;
        let mut nodes = Vec::new();
        append(
            &value,
            None,
            "$".into(),
            "$".into(),
            ".".into(),
            String::new(),
            &mut nodes,
        );
        Ok(Self { value, nodes })
    }
    pub fn nodes(&self) -> &[JsonNode] {
        &self.nodes
    }
    pub fn node(&self, id: usize) -> Option<&JsonNode> {
        self.nodes.get(id)
    }
    pub fn format(&self, pretty: bool) -> String {
        serialize(&self.value, pretty)
    }
    pub fn node_value(&self, id: usize) -> Option<String> {
        self.value
            .pointer(&self.nodes.get(id)?.pointer)
            .map(|value| serialize(value, true))
    }
}

fn serialize(value: &Value, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(value)
    } else {
        serde_json::to_string(value)
    }
    .expect("serializing a parsed JSON value")
}

fn append(
    value: &Value,
    parent: Option<usize>,
    label: String,
    json_path: String,
    jq: String,
    pointer: String,
    nodes: &mut Vec<JsonNode>,
) -> usize {
    let (kind, preview) = match value {
        Value::Object(map) => (JsonKind::Object, format!("{{ {} 个属性 }}", map.len())),
        Value::Array(array) => (JsonKind::Array, format!("[ {} 项 ]", array.len())),
        Value::String(_) => (JsonKind::String, serialize(value, false)),
        Value::Number(_) => (JsonKind::Number, value.to_string()),
        Value::Bool(_) => (JsonKind::Boolean, value.to_string()),
        Value::Null => (JsonKind::Null, "null".into()),
    };
    let preview = if preview.chars().count() > 120 {
        preview.chars().take(120).collect::<String>() + "…"
    } else {
        preview
    };
    let id = nodes.len();
    nodes.push(JsonNode {
        parent,
        children: Vec::new(),
        label,
        kind,
        preview,
        json_path: json_path.clone(),
        jq: jq.clone(),
        pointer: pointer.clone(),
    });
    let mut children = Vec::new();
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let segment = format!("[{}]", quoted(key));
                let jq = if jq == "." {
                    format!(".{segment}")
                } else {
                    format!("{jq}{segment}")
                };
                children.push(append(
                    value,
                    Some(id),
                    quoted(key),
                    format!("{json_path}{segment}"),
                    jq,
                    format!("{pointer}/{}", key.replace('~', "~0").replace('/', "~1")),
                    nodes,
                ));
            }
        }
        Value::Array(array) => {
            for (index, value) in array.iter().enumerate() {
                let segment = format!("[{index}]");
                let jq = if jq == "." {
                    format!(".{segment}")
                } else {
                    format!("{jq}{segment}")
                };
                children.push(append(
                    value,
                    Some(id),
                    segment.clone(),
                    format!("{json_path}{segment}"),
                    jq,
                    format!("{pointer}/{index}"),
                    nodes,
                ));
            }
        }
        _ => {}
    }
    nodes[id].children = children;
    id
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered_nodes_have_absolute_paths_and_parent_links() {
        let doc = JsonDocument::parse(r#"{"b":1,"a":[{"name":"你好"},null]}"#).unwrap();
        let root = &doc.nodes()[0];
        assert_eq!(root.json_path, "$");
        assert_eq!(root.jq, ".");
        assert_eq!(root.children, vec![1, 2]);
        assert_eq!(doc.nodes()[1].label, "\"b\"");
        assert_eq!(doc.nodes()[4].json_path, "$[\"a\"][0][\"name\"]");
        assert_eq!(doc.nodes()[4].jq, ".[\"a\"][0][\"name\"]");
        assert_eq!(doc.nodes()[4].parent, Some(3));
        assert_eq!(doc.node_value(4).as_deref(), Some("\"你好\""));
        assert_eq!(
            doc.format(true),
            super::super::prettify(r#"{"b":1,"a":[{"name":"你好"},null]}"#).unwrap()
        );
    }
    #[test]
    fn special_keys_are_quoted_and_pointer_escaped() {
        let input = r#"{"a.b":{"q\"uote":{"":{ "~/\n\\(x)":42}}}}"#;
        let doc = JsonDocument::parse(input).unwrap();
        let leaf = doc.nodes().last().unwrap();
        assert_eq!(
            leaf.json_path,
            "$[\"a.b\"][\"q\\\"uote\"][\"\"][\"~/\\n\\\\(x)\"]"
        );
        assert_eq!(leaf.jq, ".[\"a.b\"][\"q\\\"uote\"][\"\"][\"~/\\n\\\\(x)\"]");
        assert_eq!(doc.node_value(doc.nodes().len() - 1).as_deref(), Some("42"));
        assert!(doc.node_value(usize::MAX).is_none());
    }
    #[test]
    fn scalars_and_empty_containers_are_inspectable() {
        for input in ["null", "true", "123", "\"😀\"", "{}", "[]"] {
            let doc = JsonDocument::parse(input).unwrap();
            assert_eq!(doc.nodes().len(), 1);
            assert!(doc.nodes()[0].children.is_empty());
            assert_eq!(doc.node_value(0).as_deref(), Some(input));
        }
        for input in ["", "[1,]", "{} true"] {
            assert!(JsonDocument::parse(input).is_err());
        }
    }
}
