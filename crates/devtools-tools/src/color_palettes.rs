//! Generate simple palettes from a seed color (offline).

use crate::color_tool::{self, Rgb};
use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::COLOR_PALETTES,
        name: "Color Palettes",
        category: Category::Graphics,
        summary: "从种子色生成互补 / 类似 / 三元组 / 单色阶。输入 #hex / rgb / hsl。",
        keywords: &["palette", "color", "complementary", "色板", "配色"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let rows = color_tool::convert_all(input)?;
    let hex = rows
        .iter()
        .find(|(k, _)| *k == "HEX")
        .map(|(_, v)| v.clone())
        .ok_or_else(|| "无法解析颜色".to_string())?;
    let rgb = parse_hex_rgb(&hex)?;
    let (h, s, l) = rgb_to_hsl(rgb);

    let mut out = vec![("种子", hex.clone())];
    out.push(("互补", hsl_to_hex((h + 180.0) % 360.0, s, l)));
    out.push(("类似 −30°", hsl_to_hex((h + 330.0) % 360.0, s, l)));
    out.push(("类似 +30°", hsl_to_hex((h + 30.0) % 360.0, s, l)));
    out.push(("三元 +120°", hsl_to_hex((h + 120.0) % 360.0, s, l)));
    out.push(("三元 +240°", hsl_to_hex((h + 240.0) % 360.0, s, l)));
    for (i, factor) in [0.2_f64, 0.35, 0.5, 0.65, 0.8].into_iter().enumerate() {
        out.push((
            label_mono(i),
            hsl_to_hex(h, s, factor.clamp(0.05, 0.95)),
        ));
    }
    Ok(out)
}

fn label_mono(i: usize) -> &'static str {
    ["单色 1", "单色 2", "单色 3", "单色 4", "单色 5"][i]
}

fn parse_hex_rgb(hex: &str) -> Result<Rgb, String> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 {
        return Err("内部 HEX 无效".into());
    }
    let n = u32::from_str_radix(s, 16).map_err(|_| "HEX 解析失败")?;
    Ok(Rgb {
        r: ((n >> 16) & 0xff) as u8,
        g: ((n >> 8) & 0xff) as u8,
        b: (n & 0xff) as u8,
    })
}

fn rgb_to_hsl(rgb: Rgb) -> (f64, f64, f64) {
    let r = rgb.r as f64 / 255.0;
    let g = rgb.g as f64 / 255.0;
    let b = rgb.b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if (max - min).abs() < f64::EPSILON {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if (max - r).abs() < f64::EPSILON {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if (max - g).abs() < f64::EPSILON {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    } / 6.0;
    (h * 360.0, s, l)
}

fn hsl_to_hex(h: f64, s: f64, l: f64) -> String {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let r = ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let g = ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    let b = ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    format!("#{r:02X}{g:02X}{b:02X}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_from_hex() {
        let rows = convert_all("#3366cc").unwrap();
        assert!(rows.len() >= 8);
        assert!(rows[0].1.starts_with('#'));
    }
}
