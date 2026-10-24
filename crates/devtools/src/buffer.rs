//! UTF-8 editing state; no GPUI dependency. Undo is linear and bounded.
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

const UNDO_LIMIT: usize = 128;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Snapshot {
    text: String,
    anchor: usize,
    cursor: usize,
}

pub struct Buffer {
    state: Snapshot,
    multiline: bool,
    pub marked: Option<Range<usize>>,
    undo: Vec<Snapshot>,
    composition: Option<Snapshot>,
}

pub fn from_utf16(text: &str, offset: usize) -> usize {
    let mut units = 0;
    for (byte, ch) in text.char_indices() {
        if units + ch.len_utf16() > offset {
            return byte;
        }
        units += ch.len_utf16();
    }
    text.len()
}

impl Buffer {
    pub fn new(multiline: bool) -> Self {
        Self {
            state: Snapshot {
                text: String::new(),
                anchor: 0,
                cursor: 0,
            },
            multiline,
            marked: None,
            undo: Vec::new(),
            composition: None,
        }
    }

    pub fn text(&self) -> &str {
        &self.state.text
    }
    pub fn cursor(&self) -> usize {
        self.state.cursor
    }
    pub fn reversed(&self) -> bool {
        self.state.cursor < self.state.anchor
    }
    pub fn selection(&self) -> Range<usize> {
        self.state.anchor.min(self.cursor())..self.state.anchor.max(self.cursor())
    }
    pub fn to_utf16(&self, byte: usize) -> usize {
        self.text()[..self.boundary(byte)].encode_utf16().count()
    }
    pub fn range_from_utf16(&self, range: Range<usize>) -> Range<usize> {
        from_utf16(self.text(), range.start)..from_utf16(self.text(), range.end.max(range.start))
    }
    pub fn range_to_utf16(&self, range: Range<usize>) -> Range<usize> {
        self.to_utf16(range.start)..self.to_utf16(range.end)
    }
    fn boundary(&self, byte: usize) -> usize {
        self.text().floor_char_boundary(byte.min(self.text().len()))
    }
    fn remember(&mut self) {
        if self.undo.len() == UNDO_LIMIT {
            self.undo.remove(0);
        }
        self.undo.push(self.state.clone());
    }
    fn finish_composition(&mut self) {
        if let Some(baseline) = self.composition.take()
            && baseline.text == self.state.text
        {
            self.undo.pop();
        }
        self.marked = None;
    }
    pub fn unmark(&mut self) {
        self.finish_composition();
    }
    pub fn place_cursor_without_undo(&mut self, byte: usize, extend: bool) {
        // Movement neither adds nor removes history, including an IME snapshot.
        self.marked = None;
        self.composition = None;
        self.state.cursor = self.boundary(byte);
        if !extend {
            self.state.anchor = self.cursor();
        }
    }
    pub fn select_all(&mut self) {
        self.place_cursor_without_undo(0, false);
        self.place_cursor_without_undo(self.text().len(), true);
    }
    pub fn previous(&self) -> usize {
        self.text()
            .grapheme_indices(true)
            .rev()
            .find_map(|(i, _)| (i < self.cursor()).then_some(i))
            .unwrap_or(0)
    }
    pub fn next(&self) -> usize {
        self.text()
            .grapheme_indices(true)
            .find_map(|(i, _)| (i > self.cursor()).then_some(i))
            .unwrap_or(self.text().len())
    }
    pub fn line_range(&self) -> Range<usize> {
        let start = self.text()[..self.cursor()]
            .rfind('\n')
            .map_or(0, |i| i + 1);
        let end = self.text()[self.cursor()..]
            .find('\n')
            .map_or(self.text().len(), |i| i + self.cursor());
        start..end
    }
    pub fn cursor_line(&self) -> usize {
        self.text()[..self.cursor()]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
    }

    /// Range is UTF-8. IME selection is relative to the inserted text in UTF-16.
    pub fn replace(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        mark: bool,
        selected: Option<Range<usize>>,
    ) -> bool {
        let normalized = if self.multiline {
            text.replace("\r\n", "\n").replace('\r', "\n")
        } else {
            text.replace(['\r', '\n'], "")
        };
        // A newline-only paste in a line field must not delete a selection.
        if !self.multiline && !text.is_empty() && normalized.is_empty() {
            return false;
        }
        let range = range
            .or_else(|| self.marked.clone())
            .unwrap_or_else(|| self.selection());
        let start = self.boundary(range.start);
        let end = self.boundary(range.end.max(start));
        let changed = self.text()[start..end] != normalized;
        if mark {
            if self.composition.is_none() {
                self.composition = Some(self.state.clone());
                self.remember();
            }
        } else if self.composition.is_none() && changed {
            self.remember();
        }
        self.state.text.replace_range(start..end, &normalized);
        let inserted_end = start + normalized.len();
        let selection = if normalized == text { selected } else { None };
        let selection = selection
            .map(|r| {
                start + from_utf16(&normalized, r.start)
                    ..start + from_utf16(&normalized, r.end.max(r.start))
            })
            .unwrap_or(inserted_end..inserted_end);
        self.state.anchor = selection.start;
        self.state.cursor = selection.end;
        self.marked = (mark && !normalized.is_empty()).then_some(start..inserted_end);
        if !mark || normalized.is_empty() {
            self.finish_composition();
        }
        changed
    }
    pub fn undo(&mut self) -> bool {
        self.marked = None;
        self.composition = None;
        if let Some(state) = self.undo.pop() {
            self.state = state;
            true
        } else {
            false
        }
    }
    pub fn set_multiline(&mut self, multiline: bool) {
        if self.multiline == multiline {
            return;
        }
        self.multiline = multiline;
        if !multiline && self.state.text.chars().any(|c| c == '\n' || c == '\r') {
            let cleaned = self.state.text.replace(['\r', '\n'], "");
            let cursor = self.boundary(self.state.cursor.min(cleaned.len()));
            let anchor = self.boundary(self.state.anchor.min(cleaned.len()));
            self.state.text = cleaned;
            self.state.cursor = cursor;
            self.state.anchor = anchor;
            self.marked = None;
            self.composition = None;
        }
    }

    /// Used only to publish stored output, not for user edits.
    pub fn set_text(&mut self, text: &str) {
        self.state = Snapshot {
            text: text.to_owned(),
            anchor: 0,
            cursor: 0,
        };
        self.marked = None;
        self.composition = None;
        self.undo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn grapheme_edit_and_selection_restore() {
        let mut b = Buffer::new(true);
        b.replace(None, "你e\u{301}👩‍💻\n", false, None);
        b.place_cursor_without_undo(b.previous(), false);
        let end = b.cursor();
        let start = b.previous();
        b.place_cursor_without_undo(start, true);
        b.replace(None, "", false, None);
        assert_eq!(b.text(), "你e\u{301}\n");
        assert!(b.undo());
        assert_eq!(b.text(), "你e\u{301}👩‍💻\n");
        assert_eq!(b.selection(), start..end);
        assert!(b.reversed());
    }
    #[test]
    fn newline_normalization_and_empty_replacement() {
        let mut b = Buffer::new(true);
        b.replace(None, "a\r\nb\rc\n", false, None);
        assert_eq!(b.text(), "a\nb\nc\n");
        assert_eq!(b.text().split('\n').count(), 4);
        let mut b = Buffer::new(false);
        b.replace(None, "ab", false, None);
        b.select_all();
        assert!(!b.replace(None, "\r\n", false, None));
        assert_eq!(b.selection(), 0..2);
        b.replace(None, "x\r\ny", false, Some(0..1));
        assert_eq!(b.text(), "xy");
        assert_eq!(b.cursor(), 2);
        b.select_all();
        b.replace(None, "", false, None);
        assert_eq!(b.text(), "");
    }
    #[test]
    fn utf16_relative_composition_is_one_undo() {
        let mut b = Buffer::new(true);
        b.replace(None, "前缀:", false, None);
        b.replace(None, "😀你", true, Some(2..3));
        assert_eq!(b.selection(), "前缀:😀".len().."前缀:😀你".len());
        b.replace(None, "😀你好", true, Some(4..4));
        b.replace(None, "😀你好", false, None);
        assert_eq!(b.undo.len(), 2);
        b.undo();
        assert_eq!(b.text(), "前缀:");
        assert_eq!(from_utf16("😀你", 1), 0);
        assert_eq!(from_utf16("😀你", 2), 4);
        assert_eq!(b.range_from_utf16(0..2), 0..6);
    }
    #[test]
    fn ime_cancel_compares_own_baseline_even_when_history_full() {
        let mut b = Buffer::new(true);
        for _ in 0..140 {
            b.replace(None, "a", false, None);
        }
        assert_eq!(b.undo.len(), UNDO_LIMIT);
        b.replace(None, "临时", true, None);
        b.replace(None, "", true, None);
        assert_eq!(b.text().len(), 140);
        assert_eq!(b.undo.len(), UNDO_LIMIT - 1);
        b.select_all();
        b.replace(None, "替换", true, None);
        b.replace(None, "", true, None);
        assert_eq!(b.text(), "");
        b.undo();
        assert_eq!(b.text().len(), 140);
    }
    #[test]
    fn moving_cursor_never_changes_undo_history() {
        let mut b = Buffer::new(true);
        b.replace(None, "abc", false, None);
        b.replace(None, "中", true, None);
        let count = b.undo.len();
        b.place_cursor_without_undo(0, false);
        assert_eq!(b.undo.len(), count);
        assert!(b.marked.is_none());
        b.undo();
        assert_eq!(b.text(), "abc");
    }
    #[test]
    fn undo_cap_and_line_edges() {
        let mut b = Buffer::new(true);
        for _ in 0..140 {
            b.replace(None, "x", false, None);
        }
        for _ in 0..128 {
            assert!(b.undo());
        }
        assert!(!b.undo());
        assert_eq!(b.text().len(), 12);
        b.set_text("a\n\n末\n");
        b.place_cursor_without_undo(2, false);
        assert_eq!(b.line_range(), 2..2);
        assert_eq!(b.cursor_line(), 1);
        b.place_cursor_without_undo(usize::MAX, false);
        assert_eq!(b.line_range(), 7..7);
    }

    #[test]
    fn set_multiline_strips_newlines() {
        let mut b = Buffer::new(true);
        b.replace(None, "a\nb\rc", false, None);
        assert_eq!(b.text(), "a\nb\nc");
        b.set_multiline(false);
        assert_eq!(b.text(), "abc");
        b.select_all();
        b.replace(None, "x\ny", false, None);
        assert_eq!(b.text(), "xy");
        b.set_multiline(true);
        b.replace(None, "\n", false, None);
        assert_eq!(b.text(), "xy\n");
    }
}
