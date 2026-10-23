//! JSON tree interaction state; deliberately independent of GPUI.
use devtools_tools::json::JsonDocument;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct TreeState {
    pub selected: usize,
    pub scope: usize,
    collapsed: BTreeSet<usize>,
}

impl TreeState {
    pub fn reset(&mut self, doc: &JsonDocument) {
        self.selected = 0;
        self.scope = 0;
        self.collapse_all(doc);
    }
    pub fn expanded(&self, id: usize) -> bool {
        !self.collapsed.contains(&id)
    }
    pub fn toggle(&mut self, id: usize) {
        if !self.collapsed.remove(&id) {
            self.collapsed.insert(id);
        }
    }
    pub fn expand_all(&mut self) {
        self.collapsed.clear();
    }
    pub fn collapse_all(&mut self, doc: &JsonDocument) {
        self.collapsed = doc
            .nodes()
            .iter()
            .enumerate()
            .filter(|(id, node)| *id != self.scope && !node.children.is_empty())
            .map(|(id, _)| id)
            .collect();
        self.collapsed.remove(&self.scope);
        self.selected = self.scope;
    }
    pub fn visible(&self, doc: &JsonDocument) -> Vec<(usize, usize)> {
        let mut rows = Vec::new();
        let mut pending = vec![(self.scope, 0)];
        while let Some((id, depth)) = pending.pop() {
            let Some(node) = doc.node(id) else {
                continue;
            };
            rows.push((id, depth));
            if self.expanded(id) {
                pending.extend(node.children.iter().rev().map(|child| (*child, depth + 1)));
            }
        }
        rows
    }
    pub fn step(&mut self, down: bool, doc: &JsonDocument) {
        let rows = self.visible(doc);
        let current = rows
            .iter()
            .position(|(id, _)| *id == self.selected)
            .unwrap_or(0);
        let index = if down {
            (current + 1).min(rows.len().saturating_sub(1))
        } else {
            current.saturating_sub(1)
        };
        if let Some((id, _)) = rows.get(index) {
            self.selected = *id;
        }
    }
    pub fn right(&mut self, doc: &JsonDocument) {
        if let Some(node) = doc.node(self.selected)
            && !node.children.is_empty()
        {
            if self.expanded(self.selected) {
                self.selected = node.children[0];
            } else {
                self.collapsed.remove(&self.selected);
            }
        }
    }
    pub fn left(&mut self, doc: &JsonDocument) {
        if let Some(node) = doc.node(self.selected) {
            if !node.children.is_empty() && self.expanded(self.selected) {
                self.collapsed.insert(self.selected);
            } else if self.selected != self.scope
                && let Some(parent) = node.parent
            {
                self.selected = parent;
            }
        }
    }
    pub fn focus_selected(&mut self, doc: &JsonDocument) {
        if doc
            .node(self.selected)
            .is_some_and(|node| node.kind.is_container())
        {
            self.scope = self.selected;
            self.collapsed.remove(&self.scope);
        }
    }
    pub fn root(&mut self) {
        self.scope = 0;
        self.selected = 0;
        self.collapsed.remove(&0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn expand_select_and_focus_preserve_absolute_paths() {
        let doc = JsonDocument::parse(r#"{"a":[{"x":1}],"b":true}"#).unwrap();
        let mut state = TreeState::default();
        state.reset(&doc);
        assert_eq!(state.visible(&doc), vec![(0, 0), (1, 1), (4, 1)]);
        state.step(true, &doc);
        state.right(&doc);
        assert_eq!(state.selected, 1);
        state.right(&doc);
        assert_eq!(state.selected, 2);
        state.focus_selected(&doc);
        assert_eq!(state.visible(&doc), vec![(2, 0), (3, 1)]);
        assert_eq!(doc.node(3).unwrap().jq, ".[\"a\"][0][\"x\"]");
        state.step(true, &doc);
        state.left(&doc);
        assert_eq!(state.selected, 2);
        state.root();
        state.expand_all();
        assert_eq!(state.visible(&doc).len(), 5);
        state.collapse_all(&doc);
        assert_eq!(state.selected, 0);
        assert_eq!(state.visible(&doc).len(), 3);
    }
    #[test]
    fn root_can_collapse_and_empty_document_has_stable_selection() {
        let doc = JsonDocument::parse("{}").unwrap();
        let mut state = TreeState::default();
        state.reset(&doc);
        state.step(true, &doc);
        state.step(false, &doc);
        state.left(&doc);
        state.right(&doc);
        assert_eq!(state.selected, 0);
        assert_eq!(state.visible(&doc), vec![(0, 0)]);
        let doc = JsonDocument::parse("[1,2]").unwrap();
        state.reset(&doc);
        state.toggle(0);
        assert_eq!(state.visible(&doc), vec![(0, 0)]);
        state.right(&doc);
        assert_eq!(state.visible(&doc).len(), 3);
    }
}
