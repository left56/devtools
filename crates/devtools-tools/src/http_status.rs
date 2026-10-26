//! HTTP status code reference (static table).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::HTTP_STATUS,
        name: "HTTP status",
        category: Category::Web,
        summary: "HTTP 状态码查询：输入数字或关键词。",
        keywords: &["http", "status", "404", "状态码"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const CODES: &[(u16, &str, &str)] = &[
    (100, "Continue", "继续"),
    (101, "Switching Protocols", "切换协议"),
    (200, "OK", "成功"),
    (201, "Created", "已创建"),
    (202, "Accepted", "已接受"),
    (204, "No Content", "无内容"),
    (301, "Moved Permanently", "永久重定向"),
    (302, "Found", "临时重定向"),
    (304, "Not Modified", "未修改"),
    (400, "Bad Request", "错误请求"),
    (401, "Unauthorized", "未认证"),
    (403, "Forbidden", "禁止访问"),
    (404, "Not Found", "未找到"),
    (405, "Method Not Allowed", "方法不允许"),
    (409, "Conflict", "冲突"),
    (410, "Gone", "已失效"),
    (418, "I'm a teapot", "茶壶"),
    (429, "Too Many Requests", "请求过多"),
    (500, "Internal Server Error", "服务器错误"),
    (502, "Bad Gateway", "网关错误"),
    (503, "Service Unavailable", "服务不可用"),
    (504, "Gateway Timeout", "网关超时"),
];

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入状态码或关键词，例如 404 / not found".into());
    }
    if let Ok(code) = t.parse::<u16>() {
        let row = CODES
            .iter()
            .find(|(c, _, _)| *c == code)
            .ok_or_else(|| format!("未收录状态码 {code}"))?;
        return Ok(vec![
            ("状态码", row.0.to_string()),
            ("英文", row.1.to_string()),
            ("中文", row.2.to_string()),
            ("类别", category(row.0)),
        ]);
    }
    let q = t.to_ascii_lowercase();
    let hits: Vec<_> = CODES
        .iter()
        .filter(|(_, en, zh)| en.to_ascii_lowercase().contains(&q) || zh.contains(t))
        .collect();
    if hits.is_empty() {
        return Err("没有匹配的状态码".into());
    }
    let mut rows = Vec::new();
    for (i, (code, en, zh)) in hits.into_iter().take(8).enumerate() {
        let label: &'static str = match i {
            0 => "匹配 1",
            1 => "匹配 2",
            2 => "匹配 3",
            3 => "匹配 4",
            4 => "匹配 5",
            5 => "匹配 6",
            6 => "匹配 7",
            _ => "匹配 8",
        };
        rows.push((label, format!("{code} {en}（{zh}）")));
    }
    Ok(rows)
}

fn category(code: u16) -> String {
    match code / 100 {
        1 => "1xx 信息".into(),
        2 => "2xx 成功".into(),
        3 => "3xx 重定向".into(),
        4 => "4xx 客户端错误".into(),
        5 => "5xx 服务端错误".into(),
        _ => "其他".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn by_code_and_text() {
        assert_eq!(convert_all("404").unwrap()[1].1, "Not Found");
        assert!(!convert_all("not found").unwrap().is_empty());
    }
}
