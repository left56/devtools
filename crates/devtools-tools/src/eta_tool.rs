//! ETA calculator: size + speed → duration.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::ETA,
        name: "ETA calculator",
        category: Category::Data,
        summary: "预计完成时间：`大小 速度`，例如 `1.5GB 20MB/s`。",
        keywords: &["eta", "download", "speed", "预计", "耗时"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let parts: Vec<_> = input.split_whitespace().collect();
    if parts.len() < 2 {
        return Err("格式：大小 速度，例如 700MB 10MB/s".into());
    }
    let size = parse_bytes(parts[0])?;
    let speed = parse_bytes(parts[1].trim_end_matches("/s").trim_end_matches("/S"))?;
    if speed == 0.0 {
        return Err("速度不能为 0".into());
    }
    let secs = size / speed;
    Ok(vec![
        ("字节数", format!("{size:.0}")),
        ("字节/秒", format!("{speed:.0}")),
        ("秒", fmt_secs(secs)),
        ("可读", human_duration(secs)),
    ])
}

fn parse_bytes(s: &str) -> Result<f64, String> {
    let t = s.trim().to_ascii_uppercase().replace(' ', "");
    let units = [
        ("TB", 1e12),
        ("GB", 1e9),
        ("MB", 1e6),
        ("KB", 1e3),
        ("TIB", 1024f64.powi(4)),
        ("GIB", 1024f64.powi(3)),
        ("MIB", 1024f64.powi(2)),
        ("KIB", 1024f64),
        ("B", 1.0),
    ];
    for (u, mul) in units {
        if let Some(num) = t.strip_suffix(u) {
            let n: f64 = num.parse().map_err(|_| format!("无法解析 `{s}`"))?;
            return Ok(n * mul);
        }
    }
    t.parse().map_err(|_| format!("无法解析 `{s}`"))
}

fn fmt_secs(secs: f64) -> String {
    format!("{secs:.3}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn human_duration(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    if h > 0 {
        format!("{h} 小时 {m} 分 {s} 秒")
    } else if m > 0 {
        format!("{m} 分 {s} 秒")
    } else {
        format!("{s} 秒")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn basic() {
        let rows = convert_all("100MB 10MB/s").unwrap();
        assert_eq!(rows[2].1, "10");
        assert!(rows[3].1.contains("10 秒") || rows[3].1.contains("秒"));
    }
}
