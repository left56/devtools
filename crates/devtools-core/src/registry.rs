use std::collections::BTreeSet;

use crate::{Category, ToolMeta};

/// The tools the shell is allowed to show.
///
/// Constructing a registry rejects duplicate ids. Planned ids that have no
/// implementation must not be inserted.
#[derive(Debug, Clone)]
pub struct Registry {
    tools: Vec<ToolMeta>,
}

impl Registry {
    /// # Panics
    ///
    /// Panics if two tools share an id. That is a programming error in the
    /// builtin catalog, not a user input failure.
    pub fn new(tools: Vec<ToolMeta>) -> Self {
        let mut seen = BTreeSet::new();
        for tool in &tools {
            assert!(
                seen.insert(tool.id),
                "duplicate tool id `{}` in the catalog",
                tool.id
            );
        }
        Self { tools }
    }

    pub fn tools(&self) -> &[ToolMeta] {
        &self.tools
    }

    pub fn contains(&self, id: &str) -> bool {
        self.get(id).is_some()
    }

    pub fn get(&self, id: &str) -> Option<&ToolMeta> {
        self.tools.iter().find(|tool| tool.id == id)
    }

    pub fn categories_in_use(&self) -> Vec<Category> {
        Category::ALL
            .into_iter()
            .filter(|category| self.tools.iter().any(|tool| tool.category == *category))
            .collect()
    }

    /// Case-insensitive match on id, name, summary, keywords, and category label.
    ///
    /// An empty query matches everything in `category`. `None` means all
    /// categories. Order is the catalog order.
    pub fn search<'a>(&'a self, query: &str, category: Option<Category>) -> Vec<&'a ToolMeta> {
        let query = query.trim().to_lowercase();
        self.tools
            .iter()
            .filter(|tool| category.is_none_or(|expected| tool.category == expected))
            .filter(|tool| query.is_empty() || matches_query(tool, &query))
            .collect()
    }
}

fn matches_query(tool: &ToolMeta, query: &str) -> bool {
    let mut fields = vec![
        tool.id.to_lowercase(),
        tool.name.to_lowercase(),
        tool.summary.to_lowercase(),
        tool.category.label().to_lowercase(),
        tool.category.id().to_lowercase(),
    ];
    fields.extend(tool.keywords.iter().map(|keyword| keyword.to_lowercase()));
    fields.iter().any(|field| field.contains(query))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Availability, NetworkUse, ids};

    fn sample(id: &'static str, name: &'static str, category: Category) -> ToolMeta {
        ToolMeta {
            id,
            name,
            category,
            summary: "示例",
            keywords: &["alpha"],
            network: NetworkUse::Offline,
            availability: Availability::LogicOnly,
        }
    }

    #[test]
    fn search_filters_by_query_and_category() {
        let registry = Registry::new(vec![
            sample(ids::JSON, "JSON", Category::Data),
            sample(ids::TIMESTAMP, "Timestamp", Category::Time),
        ]);

        let names: Vec<_> = registry
            .search("json", None)
            .into_iter()
            .map(|tool| tool.id)
            .collect();
        assert_eq!(names, vec![ids::JSON]);

        let timed: Vec<_> = registry
            .search("", Some(Category::Time))
            .into_iter()
            .map(|tool| tool.id)
            .collect();
        assert_eq!(timed, vec![ids::TIMESTAMP]);
        assert_eq!(registry.search("不存在", None).len(), 0);
    }

    #[test]
    fn empty_categories_are_hidden() {
        let registry = Registry::new(vec![sample(ids::JSON, "JSON", Category::Data)]);
        assert_eq!(registry.categories_in_use(), vec![Category::Data]);
    }

    #[test]
    #[should_panic(expected = "duplicate tool id `json`")]
    fn duplicate_ids_panic() {
        let _registry = Registry::new(vec![
            sample(ids::JSON, "JSON", Category::Data),
            sample(ids::JSON, "JSON again", Category::Data),
        ]);
    }
}
