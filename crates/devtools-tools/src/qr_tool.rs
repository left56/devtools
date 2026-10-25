//! QR code → SVG + ASCII (offline via qrcode crate).

use qrcode::QrCode;
use qrcode::render::svg;
use qrcode::types::EcLevel;
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::QR_CODE,
        name: "QR code",
        category: Category::Generators,
        summary: "文本生成 QR：界面预览 SVG，并给出可复制 ASCII。离线。",
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
    // Prefer ASCII #/space over U+2588 full block: on macOS Menlo/SF Mono the
    // block glyph is often wider than a space, so the grid skews. Keep a quiet
    // zone so scanners still read printed ASCII copies.
    let ascii = code
        .render::<char>()
        .quiet_zone(true)
        .module_dimensions(2, 1)
        .dark_color('#')
        .light_color(' ')
        .build();
    Ok(vec![
        ("SVG", svg),
        ("ASCII", ascii),
        ("内容长度", t.chars().count().to_string()),
        ("纠错", "M".into()),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn makes_svg() {
        let rows = convert_all("https://example.com").unwrap();
        assert!(rows[0].1.contains("<svg"));
        assert!(!rows[1].1.is_empty());
    }
}
