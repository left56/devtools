//! Line-level text diff. Pure Rust — no external `diff` process.

use devtools_core::{Availability, Category, NetworkUse, ToolMeta, ids};
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Equal,
    Insert,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    pub kind: LineKind,
    pub text: String,
    /// 1-based line number in the left (old) text, when applicable.
    pub old_line: Option<usize>,
    /// 1-based line number in the right (new) text, when applicable.
    pub new_line: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffResult {
    pub lines: Vec<DiffLine>,
    pub inserts: usize,
    pub deletes: usize,
}

pub fn meta() -> ToolMeta {
    ToolMeta {
        id: ids::DIFF,
        name: "Diff",
        category: Category::Text,
        summary: "行级文本 diff。插入与删除高亮，不调用外部 diff 进程。",
        keywords: &["diff", "compare", "对比", "差异", "patch"],
        network: NetworkUse::Offline,
        availability: Availability::Ready,
    }
}

/// Diff `left` (old) against `right` (new) line by line.
pub fn line_diff(left: &str, right: &str) -> DiffResult {
    let diff = TextDiff::from_lines(left, right);
    let mut lines = Vec::new();
    let mut inserts = 0usize;
    let mut deletes = 0usize;
    let mut old_no = 1usize;
    let mut new_no = 1usize;

    for change in diff.iter_all_changes() {
        let text = change.to_string().trim_end_matches('\n').to_owned();
        match change.tag() {
            ChangeTag::Equal => {
                lines.push(DiffLine {
                    kind: LineKind::Equal,
                    text,
                    old_line: Some(old_no),
                    new_line: Some(new_no),
                });
                old_no += 1;
                new_no += 1;
            }
            ChangeTag::Delete => {
                deletes += 1;
                lines.push(DiffLine {
                    kind: LineKind::Delete,
                    text,
                    old_line: Some(old_no),
                    new_line: None,
                });
                old_no += 1;
            }
            ChangeTag::Insert => {
                inserts += 1;
                lines.push(DiffLine {
                    kind: LineKind::Insert,
                    text,
                    old_line: None,
                    new_line: Some(new_no),
                });
                new_no += 1;
            }
        }
    }

    DiffResult {
        lines,
        inserts,
        deletes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_is_all_equal() {
        let out = line_diff("a\nb\n", "a\nb\n");
        assert_eq!(out.inserts, 0);
        assert_eq!(out.deletes, 0);
        assert!(out.lines.iter().all(|l| l.kind == LineKind::Equal));
    }

    #[test]
    fn insert_and_delete() {
        let out = line_diff("a\nb\nc\n", "a\nx\nc\n");
        assert_eq!(out.deletes, 1);
        assert_eq!(out.inserts, 1);
        assert!(out.lines.iter().any(|l| l.kind == LineKind::Delete && l.text == "b"));
        assert!(out.lines.iter().any(|l| l.kind == LineKind::Insert && l.text == "x"));
    }

    #[test]
    fn empty_both() {
        let out = line_diff("", "");
        assert!(out.lines.is_empty() || out.lines.iter().all(|l| l.text.is_empty()));
        assert_eq!(out.inserts + out.deletes, 0);
    }

    #[test]
    fn added_file() {
        let out = line_diff("", "hello\n");
        assert_eq!(out.inserts, 1);
        assert_eq!(out.deletes, 0);
    }
}
