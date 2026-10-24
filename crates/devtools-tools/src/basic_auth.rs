//! HTTP Basic Auth header from user:password.

use base64::Engine;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::BASIC_AUTH,
        name: "Basic Auth",
        category: Category::Web,
        summary: "从用户名密码生成 HTTP Basic Authorization 头。",
        keywords: &["basic", "auth", "authorization", "http", "认证"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空，格式：user:password".into());
    }
    let (user, pass) = if let Some((u, p)) = t.split_once(':') {
        (u, p)
    } else {
        return Err("格式应为 user:password".into());
    };
    let token = base64::engine::general_purpose::STANDARD.encode(format!("{user}:{pass}"));
    let header = format!("Basic {token}");
    Ok(vec![
        ("用户名", user.to_string()),
        ("密码", pass.to_string()),
        ("Base64", token),
        ("Authorization", header.clone()),
        ("Header 行", format!("Authorization: {header}")),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typical() {
        let rows = convert_all("Aladdin:open sesame").unwrap();
        assert_eq!(rows[2].1, "QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
        assert!(rows[3].1.starts_with("Basic "));
    }
}
