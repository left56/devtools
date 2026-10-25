//! Wi‑Fi QR payload (WIFI:T:...;;) then reuse QR renderer.

use crate::qr_tool;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::WIFI_QR,
        name: "Wi‑Fi QR",
        category: Category::Generators,
        summary: "生成 WIFI: 连接串与 SVG 预览。输入：SSID\\n加密\\n密码（加密=WPA/WEP/nopass）。无 ASCII。",
        keywords: &["wifi", "qr", "wpa", "无线"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let lines: Vec<_> = input.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return Err("格式：第1行 SSID，第2行 WPA/WEP/nopass，第3行密码".into());
    }
    let ssid = lines[0];
    let auth = lines.get(1).copied().unwrap_or("WPA");
    let pass = lines.get(2).copied().unwrap_or("");
    let auth_u = auth.to_ascii_uppercase();
    let t = match auth_u.as_str() {
        "WPA" | "WPA2" | "WEP" | "NOPASS" => {
            if auth_u == "NOPASS" { "nopass" } else { auth }
        }
        _ => auth,
    };
    let payload = format!(
        "WIFI:T:{};S:{};P:{};;",
        escape(t),
        escape(ssid),
        escape(pass)
    );
    let mut rows = vec![("WIFI 字符串", payload.clone())];
    let qr_rows = qr_tool::convert_all(&payload)?;
    rows.extend(qr_rows);
    Ok(rows)
}

fn escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace(';', "\\;")
        .replace(',', "\\,")
        .replace('"', "\\\"")
        .replace(':', "\\:")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn builds_payload() {
        let rows = convert_all("HomeNet\nWPA\nsecret").unwrap();
        assert!(rows[0].1.contains("WIFI:T:WPA;S:HomeNet;"));
        assert!(rows.iter().any(|r| r.0 == "SVG" && r.1.contains("<svg")));
    }
}
