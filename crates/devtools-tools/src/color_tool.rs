//! Color converter: hex ↔ rgb ↔ hsl.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::COLOR,
        name: "Color Inspector",
        category: Category::Graphics,
        summary: "颜色检查器：预览色块、色盘点选、#hex / rgb / hsl / hsv / cmyk。",
        keywords: &["color", "hex", "rgb", "hsl", "hsv", "cmyk", "颜色", "取色", "色盘", "预览"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn to_hex(rgb: Rgb) -> String {
    format!("#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b)
}

/// Evenly spaced hues at fixed S/L — for the quick color wheel strip.
pub fn hue_wheel(count: usize, s: f64, l: f64) -> Vec<String> {
    let n = count.max(1);
    (0..n)
        .map(|i| {
            let h = (i as f64) * 360.0 / n as f64;
            to_hex(hsl_to_rgb(h, s, l))
        })
        .collect()
}

pub fn hsl_hex(h: f64, s: f64, l: f64) -> String {
    to_hex(hsl_to_rgb(h, s, l))
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let rgb = parse_color(input)?;
    let (h, s, l) = rgb_to_hsl(rgb);
    let (hv, sv, vv) = rgb_to_hsv(rgb);
    let (c, m, y, k) = rgb_to_cmyk(rgb);
    let hex = to_hex(rgb);
    let mut rows = vec![
        ("预览", hex.clone()),
        ("HEX", hex.clone()),
        ("hex", format!("#{:02x}{:02x}{:02x}", rgb.r, rgb.g, rgb.b)),
        ("RGB", format!("rgb({}, {}, {})", rgb.r, rgb.g, rgb.b)),
        (
            "HSL",
            format!(
                "hsl({}, {}%, {}%)",
                h.round() as i32,
                (s * 100.0).round() as i32,
                (l * 100.0).round() as i32
            ),
        ),
        (
            "HSV",
            format!(
                "hsv({}, {}%, {}%)",
                hv.round() as i32,
                (sv * 100.0).round() as i32,
                (vv * 100.0).round() as i32
            ),
        ),
        (
            "CMYK",
            format!(
                "cmyk({}%, {}%, {}%, {}%)",
                (c * 100.0).round() as i32,
                (m * 100.0).round() as i32,
                (y * 100.0).round() as i32,
                (k * 100.0).round() as i32
            ),
        ),
        ("R", rgb.r.to_string()),
        ("G", rgb.g.to_string()),
        ("B", rgb.b.to_string()),
        ("互补", to_hex(hsl_to_rgb((h + 180.0) % 360.0, s, l))),
    ];
    for (i, factor) in [0.9_f64, 0.75, 0.6].into_iter().enumerate() {
        rows.push((
            ["浅色 1", "浅色 2", "浅色 3"][i],
            to_hex(hsl_to_rgb(h, s, factor.clamp(0.05, 0.95))),
        ));
    }
    for (i, factor) in [0.4_f64, 0.25, 0.12].into_iter().enumerate() {
        rows.push((
            ["深色 1", "深色 2", "深色 3"][i],
            to_hex(hsl_to_rgb(h, s, factor.clamp(0.05, 0.95))),
        ));
    }
    Ok(rows)
}

/// Parse `#hex` / `rgb()` / `hsl()` into RGB.
pub fn parse_color(input: &str) -> Result<Rgb, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    if let Some(rgb) = parse_hex(t) {
        return Ok(rgb);
    }
    if let Some(rgb) = parse_rgb_fn(t) {
        return Ok(rgb);
    }
    if let Some(rgb) = parse_hsl_fn(t) {
        return Ok(rgb);
    }
    Err("无法识别：支持 #RGB/#RRGGBB、rgb(r,g,b)、hsl(h,s%,l%)".into())
}

fn parse_hex(t: &str) -> Option<Rgb> {
    let s = t.strip_prefix('#').unwrap_or(t);
    let full = match s.len() {
        3 => {
            let b = s.as_bytes();
            format!(
                "{0}{0}{1}{1}{2}{2}",
                b[0] as char, b[1] as char, b[2] as char
            )
        }
        6 => s.to_string(),
        _ => return None,
    };
    if !full.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(&full, 16).ok()?;
    Some(Rgb {
        r: ((n >> 16) & 0xFF) as u8,
        g: ((n >> 8) & 0xFF) as u8,
        b: (n & 0xFF) as u8,
    })
}

fn parse_rgb_fn(t: &str) -> Option<Rgb> {
    let lower = t.to_ascii_lowercase();
    let inner = lower.strip_prefix("rgb(")?.strip_suffix(')')?;
    let parts: Vec<_> = inner.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return None;
    }
    Some(Rgb {
        r: parts[0].parse().ok()?,
        g: parts[1].parse().ok()?,
        b: parts[2].parse().ok()?,
    })
}

fn parse_hsl_fn(t: &str) -> Option<Rgb> {
    let lower = t.to_ascii_lowercase();
    let inner = lower.strip_prefix("hsl(")?.strip_suffix(')')?;
    let parts: Vec<_> = inner.split(',').map(str::trim).collect();
    if parts.len() != 3 {
        return None;
    }
    let h: f64 = parts[0].parse().ok()?;
    let s: f64 = parts[1].trim_end_matches('%').parse().ok()?;
    let l: f64 = parts[2].trim_end_matches('%').parse().ok()?;
    Some(hsl_to_rgb(h, s / 100.0, l / 100.0))
}

fn rgb_to_hsl(rgb: Rgb) -> (f64, f64, f64) {
    let r = f64::from(rgb.r) / 255.0;
    let g = f64::from(rgb.g) / 255.0;
    let b = f64::from(rgb.b) / 255.0;
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
    };
    (h * 60.0, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Rgb {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = (h % 360.0) / 60.0;
    let x = c * (1.0 - (h_prime % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    Rgb {
        r: ((r1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        g: ((g1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
        b: ((b1 + m) * 255.0).round().clamp(0.0, 255.0) as u8,
    }
}


fn rgb_to_hsv(rgb: Rgb) -> (f64, f64, f64) {
    let r = rgb.r as f64 / 255.0;
    let g = rgb.g as f64 / 255.0;
    let b = rgb.b as f64 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let v = max;
    let d = max - min;
    let s = if max == 0.0 { 0.0 } else { d / max };
    let h = if d < f64::EPSILON {
        0.0
    } else if (max - r).abs() < f64::EPSILON {
        60.0 * (((g - b) / d) % 6.0)
    } else if (max - g).abs() < f64::EPSILON {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    (h, s, v)
}

fn rgb_to_cmyk(rgb: Rgb) -> (f64, f64, f64, f64) {
    let r = rgb.r as f64 / 255.0;
    let g = rgb.g as f64 / 255.0;
    let b = rgb.b as f64 / 255.0;
    let k = 1.0 - r.max(g).max(b);
    if (1.0 - k).abs() < f64::EPSILON {
        return (0.0, 0.0, 0.0, 1.0);
    }
    let c = (1.0 - r - k) / (1.0 - k);
    let m = (1.0 - g - k) / (1.0 - k);
    let y = (1.0 - b - k) / (1.0 - k);
    (c, m, y, k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round() {
        let rows = convert_all("#336699").unwrap();
        assert_eq!(rows.iter().find(|r| r.0 == "HEX").unwrap().1, "#336699");
        assert_eq!(
            rows.iter().find(|r| r.0 == "RGB").unwrap().1,
            "rgb(51, 102, 153)"
        );
        assert!(rows.iter().any(|r| r.0 == "浅色 1"));
    }

    #[test]
    fn rgb_input() {
        let rows = convert_all("rgb(255, 0, 0)").unwrap();
        assert_eq!(rows.iter().find(|r| r.0 == "hex").unwrap().1, "#ff0000");
    }

    #[test]
    fn short_hex() {
        let rows = convert_all("#f00").unwrap();
        assert_eq!(rows.iter().find(|r| r.0 == "HEX").unwrap().1, "#FF0000");
    }

    #[test]
    fn hue_wheel_count() {
        assert_eq!(hue_wheel(12, 0.85, 0.5).len(), 12);
    }
}
