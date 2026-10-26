//! Temperature converter (°C / °F / K).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::TEMPERATURE,
        name: "Temperature",
        category: Category::Data,
        summary: "温度换算：输入如 `25C`、`77F`、`298K`，批量显示三种单位。",
        keywords: &["temperature", "celsius", "fahrenheit", "温度"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim().replace('°', "");
    if t.is_empty() {
        return Err("输入例如 25C / 77F / 298.15K".into());
    }
    let (num_s, unit) = split_unit(&t)?;
    let value: f64 = num_s.parse().map_err(|_| format!("无法解析数字 `{num_s}`"))?;
    let celsius = match unit {
        'C' | 'c' => value,
        'F' | 'f' => (value - 32.0) * 5.0 / 9.0,
        'K' | 'k' => value - 273.15,
        _ => return Err("单位须为 C / F / K".into()),
    };
    let f = celsius * 9.0 / 5.0 + 32.0;
    let k = celsius + 273.15;
    Ok(vec![
        ("摄氏 °C", fmt(celsius)),
        ("华氏 °F", fmt(f)),
        ("开尔文 K", fmt(k)),
    ])
}

fn split_unit(t: &str) -> Result<(&str, char), String> {
    let last = t.chars().last().ok_or_else(|| "输入为空".to_string())?;
    if last.is_ascii_alphabetic() {
        Ok((t[..t.len() - last.len_utf8()].trim(), last))
    } else {
        // default Celsius
        Ok((t, 'C'))
    }
}

fn fmt(n: f64) -> String {
    format!("{n:.4}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boiling() {
        let rows = convert_all("100C").unwrap();
        assert_eq!(rows[1].1, "212");
        assert_eq!(rows[2].1, "373.15");
    }
}
