//! JSON ↔ CSV (array of objects).

use csv::{ReaderBuilder, WriterBuilder};
use serde_json::{Map, Value};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::JSON_CSV,
        name: "JSON ↔ CSV",
        category: Category::Data,
        summary: "对象数组 JSON 与 CSV 互转。自动识别输入。",
        keywords: &["json", "csv", "表格", "转换"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    if t.starts_with('[') || t.starts_with('{') {
        let v: Value = serde_json::from_str(t).map_err(|e| format!("JSON：{e}"))?;
        let csv = json_to_csv(&v)?;
        let pretty = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?;
        return Ok(vec![("CSV", csv), ("JSON", pretty)]);
    }
    let json = csv_to_json(t)?;
    let pretty = serde_json::to_string_pretty(&json).map_err(|e| e.to_string())?;
    let csv = json_to_csv(&json)?;
    Ok(vec![("JSON", pretty), ("CSV", csv)])
}

fn json_to_csv(v: &Value) -> Result<String, String> {
    let rows = match v {
        Value::Array(arr) => arr.clone(),
        Value::Object(_) => vec![v.clone()],
        _ => return Err("需要对象数组或对象".into()),
    };
    let mut headers: Vec<String> = Vec::new();
    for row in &rows {
        if let Value::Object(map) = row {
            for k in map.keys() {
                if !headers.contains(k) {
                    headers.push(k.clone());
                }
            }
        } else {
            return Err("数组元素须为对象".into());
        }
    }
    if headers.is_empty() {
        return Err("没有列".into());
    }
    let mut wtr = WriterBuilder::new().from_writer(vec![]);
    wtr.write_record(&headers).map_err(|e| e.to_string())?;
    for row in &rows {
        let map = row.as_object().unwrap();
        let record: Vec<String> = headers
            .iter()
            .map(|h| match map.get(h) {
                Some(Value::String(s)) => s.clone(),
                Some(Value::Null) | None => String::new(),
                Some(other) => other.to_string(),
            })
            .collect();
        wtr.write_record(&record).map_err(|e| e.to_string())?;
    }
    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

fn csv_to_json(csv: &str) -> Result<Value, String> {
    let mut rdr = ReaderBuilder::new()
        .flexible(true)
        .from_reader(csv.as_bytes());
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| e.to_string())?
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut arr = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| e.to_string())?;
        let mut map = Map::new();
        for (i, h) in headers.iter().enumerate() {
            let val = record.get(i).unwrap_or("");
            map.insert(h.clone(), Value::String(val.to_string()));
        }
        arr.push(Value::Object(map));
    }
    Ok(Value::Array(arr))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_csv_round() {
        let rows = convert_all(r#"[{"a":"1","b":"2"},{"a":"3","b":"4"}]"#).unwrap();
        assert!(rows[0].1.contains("a,b") || rows[0].1.contains("a,b\n") || rows[0].1.lines().next().unwrap().contains('a'));
        let back = convert_all(&rows[0].1).unwrap();
        assert!(back[0].1.contains("\"a\""));
    }
}
