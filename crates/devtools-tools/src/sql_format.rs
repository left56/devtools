//! Light SQL prettify / minify heuristics (not a full parser).

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlMode {
    Beautify,
    Minify,
}

impl SqlMode {
    pub const ALL: [SqlMode; 2] = [SqlMode::Beautify, SqlMode::Minify];

    pub fn label(self) -> &'static str {
        match self {
            Self::Beautify => "美化",
            Self::Minify => "压缩",
        }
    }

    pub fn default_mode() -> Self {
        Self::Beautify
    }
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::SQL_FORMAT,
        name: "SQL format",
        category: Category::Data,
        summary: "轻量 SQL 美化/压缩（启发式，非整库解析器）。模式切换，默认美化。",
        keywords: &["sql", "prettify", "format", "格式化", "美化", "压缩"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

const BREAK_BEFORE: &[&str] = &[
    "SELECT", "FROM", "WHERE", "AND", "OR", "LEFT JOIN", "RIGHT JOIN", "INNER JOIN", "JOIN",
    "GROUP BY", "ORDER BY", "HAVING", "LIMIT", "OFFSET", "VALUES", "SET", "UNION", "INSERT",
    "UPDATE", "DELETE",
];

/// One selected mode → one labeled row (BatchWorkbench).
pub fn convert_mode(input: &str, mode: SqlMode) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    let value = match mode {
        SqlMode::Beautify => prettify(t),
        SqlMode::Minify => minify(t),
    };
    Ok(vec![(mode.label(), value)])
}

/// Kept for tests / callers that want both; UI uses [`convert_mode`].
pub fn convert_all(input: &str) -> Result<Vec<(&'static str, String)>, String> {
    let t = input.trim();
    if t.is_empty() {
        return Err("输入为空".into());
    }
    Ok(vec![("美化", prettify(t)), ("压缩", minify(t))])
}

fn minify(sql: &str) -> String {
    let mut out = String::new();
    let mut space = false;
    for ch in sql.chars() {
        if ch.is_whitespace() {
            space = true;
            continue;
        }
        if space && !out.is_empty() && !out.ends_with('(') && ch != ',' && ch != ')' {
            out.push(' ');
        }
        space = false;
        out.push(ch);
    }
    out
}

fn prettify(sql: &str) -> String {
    let mut s = minify(sql);
    for kw in BREAK_BEFORE {
        s = insert_before_keyword(&s, kw);
    }
    s
}

fn insert_before_keyword(sql: &str, kw: &str) -> String {
    let upper = sql.to_ascii_uppercase();
    let kw_u = kw.to_ascii_uppercase();
    let mut out = String::new();
    let mut i = 0;
    let b = sql.as_bytes();
    let u = upper.as_bytes();
    let k = kw_u.as_bytes();
    while i < b.len() {
        let boundary_before = i == 0 || !b[i - 1].is_ascii_alphanumeric();
        let fits = i + k.len() <= b.len() && u[i..i + k.len()] == *k;
        let boundary_after = !fits
            || i + k.len() == b.len()
            || !b[i + k.len()].is_ascii_alphanumeric();
        if boundary_before && fits && boundary_after {
            if !out.is_empty() && !out.ends_with('\n') {
                out.push('\n');
            }
            out.push_str(&sql[i..i + k.len()]);
            i += k.len();
        } else {
            out.push(sql[i..].chars().next().unwrap());
            i += sql[i..].chars().next().unwrap().len_utf8();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn minify_spaces() {
        let rows = convert_all("select   a  from  t  where x = 1").unwrap();
        assert!(rows[1].1.to_ascii_lowercase().contains("select a from"));
        assert!(
            rows[0].1.to_ascii_uppercase().contains("\nFROM")
                || rows[0].1.to_ascii_uppercase().contains("FROM")
        );
    }

    #[test]
    fn mode_switch_one_row() {
        let pretty = convert_mode("select * from abc", SqlMode::Beautify).unwrap();
        assert_eq!(pretty.len(), 1);
        assert_eq!(pretty[0].0, "美化");
        assert!(pretty[0].1.to_ascii_uppercase().contains("FROM"));

        let mini = convert_mode("select   *   from  abc", SqlMode::Minify).unwrap();
        assert_eq!(mini.len(), 1);
        assert_eq!(mini[0].0, "压缩");
        assert_eq!(mini[0].1.to_ascii_lowercase(), "select * from abc");
    }

    #[test]
    fn default_is_beautify() {
        assert_eq!(SqlMode::default_mode(), SqlMode::Beautify);
        assert_eq!(SqlMode::ALL[0], SqlMode::Beautify);
    }
}
