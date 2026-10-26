//! QR code → SVG preview (offline via qrcode crate).

use qrcode::QrCode;
use qrcode::render::svg;
use qrcode::types::EcLevel;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::QR_CODE,
        name: "QR code",
        category: Category::Generators,
        summary: "文本生成 QR：界面预览 SVG（可复制源码）。离线，不输出 ASCII。",
        keywords: &["qr", "qrcode", "二维码", "svg"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入要编码的文本或 URL".into());
    }
    if t.chars().count() > 800 {
        return Err("文本过长（建议 ≤ 800 字符）".into());
    }
    let code = QrCode::with_error_correction_level(t.as_bytes(), EcLevel::M)
        .map_err(|e| format!("无法生成 QR：{e}"))?;
    let svg = code
        .render::<svg::Color>()
        .min_dimensions(200, 200)
        .build();
    Ok(vec![
        ("SVG", svg),
        ("内容长度", t.chars().count().to_string()),
        ("纠错", "M".into()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn makes_svg_without_ascii() {
        let rows = convert_all("https://example.com").unwrap();
        assert!(rows.iter().any(|(l, v)| *l == "SVG" && v.contains("<svg")));
        assert!(!rows.iter().any(|(l, _)| *l == "ASCII"));
    }
}
