//! SVG placeholder image markup generator.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::SVG_PLACEHOLDER,
        name: "SVG placeholder",
        category: Category::Generators,
        summary: "生成占位 SVG。输入：宽x高，可选第三行文字，例如 320x180\\nDemo。",
        keywords: &["svg", "placeholder", "占位", "图片"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let lines: Vec<_> = input.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    if lines.is_empty() {
        return Err("输入宽x高，例如 320x180".into());
    }
    let (w, h) = parse_size(lines[0])?;
    let default_label = format!("{w}×{h}");
    let label = lines.get(1).copied().unwrap_or(default_label.as_str());
    let bg = lines.get(2).copied().unwrap_or("#e5e7eb");
    let fg = lines.get(3).copied().unwrap_or("#6b7280");
    let svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}">
  <rect width="100%" height="100%" fill="{bg}"/>
  <text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle"
        font-family="system-ui,sans-serif" font-size="{fs}" fill="{fg}">{label}</text>
</svg>"#,
        fs = (w.min(h) / 8).max(12),
        label = xml_esc(label),
    );
    let data = format!(
        "data:image/svg+xml,{}",
        percent_encoding::utf8_percent_encode(&svg, percent_encoding::NON_ALPHANUMERIC)
    );
    Ok(vec![
        ("SVG", svg),
        ("Data URL", data),
        ("尺寸", format!("{w}×{h}")),
    ])
}

fn parse_size(s: &str) -> Result<(u32, u32), String> {
    let s = s.to_ascii_lowercase().replace('×', "x");
    let (a, b) = s
        .split_once('x')
        .ok_or_else(|| "格式：宽x高".to_string())?;
    let w: u32 = a.trim().parse().map_err(|_| "宽度无效")?;
    let h: u32 = b.trim().parse().map_err(|_| "高度无效")?;
    if w == 0 || h == 0 || w > 4096 || h > 4096 {
        return Err("尺寸须在 1–4096".into());
    }
    Ok((w, h))
}

fn xml_esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let rows = convert_all("100x50\nHi").unwrap();
        assert!(rows[0].1.contains("width=\"100\""));
        assert!(rows[0].1.contains("Hi"));
    }
}
