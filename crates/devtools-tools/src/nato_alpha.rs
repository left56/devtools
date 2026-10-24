//! NATO phonetic alphabet.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::NATO,
        name: "NATO alphabet",
        category: Category::Text,
        summary: "文本转北约音标字母（拼读）。",
        keywords: &["nato", "phonetic", "alphabet", "音标", "拼读"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const MAP: &[(char, &str)] = &[
    ('A', "Alfa"), ('B', "Bravo"), ('C', "Charlie"), ('D', "Delta"),
    ('E', "Echo"), ('F', "Foxtrot"), ('G', "Golf"), ('H', "Hotel"),
    ('I', "India"), ('J', "Juliett"), ('K', "Kilo"), ('L', "Lima"),
    ('M', "Mike"), ('N', "November"), ('O', "Oscar"), ('P', "Papa"),
    ('Q', "Quebec"), ('R', "Romeo"), ('S', "Sierra"), ('T', "Tango"),
    ('U', "Uniform"), ('V', "Victor"), ('W', "Whiskey"), ('X', "X-ray"),
    ('Y', "Yankee"), ('Z', "Zulu"),
    ('0', "Zero"), ('1', "One"), ('2', "Two"), ('3', "Three"),
    ('4', "Four"), ('5', "Five"), ('6', "Six"), ('7', "Seven"),
    ('8', "Eight"), ('9', "Nine"),
];

pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    let mut words = Vec::new();
    for ch in t.chars() {
        if ch.is_whitespace() {
            words.push("/".into());
            continue;
        }
        let up = ch.to_ascii_uppercase();
        if let Some((_, w)) = MAP.iter().find(|(c, _)| *c == up) {
            words.push((*w).to_string());
        } else {
            words.push(ch.to_string());
        }
    }
    Ok(vec![
        ("音标", words.join(" ")),
        ("仅字母", words.iter().filter(|w| *w != "/").cloned().collect::<Vec<_>>().join(" ")),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sos() {
        let rows = convert_all("SOS").unwrap();
        assert!(rows[0].1.contains("Sierra"));
        assert!(rows[0].1.contains("Oscar"));
    }
}
