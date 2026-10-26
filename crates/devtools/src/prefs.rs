//! Local preferences: recent tools and favorites.
//!
//! Files store **only** tool ids, timestamps, and a favorite flag.
//! Never persist tool inputs, outputs, URLs, or responses.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const PREFS_FILE: &str = "prefs.json";
const MAX_RECENTS: usize = 20;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct Prefs {
    #[serde(default)]
    pub favorites: Vec<String>,
    #[serde(default)]
    pub recents: Vec<RecentEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentEntry {
    pub id: String,
    /// Unix seconds when the tool was last selected.
    pub used_at: u64,
}

impl Prefs {
    pub fn from_json(text: &str) -> Result<Self, String> {
        let prefs: Prefs = serde_json::from_str(text).map_err(|e| e.to_string())?;
        prefs.validate_no_payloads()?;
        Ok(prefs.normalized())
    }

    pub fn to_json(&self) -> Result<String, String> {
        let normalized = self.normalized();
        normalized.validate_no_payloads()?;
        serde_json::to_string_pretty(&normalized).map_err(|e| e.to_string())
    }

    /// Drop unknown shapes that look like payloads (defensive).
    fn validate_no_payloads(&self) -> Result<(), String> {
        for id in &self.favorites {
            if id.contains('\n') || id.len() > 64 {
                return Err("favorites must be short tool ids only".into());
            }
        }
        for entry in &self.recents {
            if entry.id.contains('\n') || entry.id.len() > 64 {
                return Err("recents must be short tool ids only".into());
            }
        }
        Ok(())
    }

    fn normalized(&self) -> Self {
        let mut favorites: Vec<String> = self
            .favorites
            .iter()
            .map(|id| data_format_remap(id).to_owned())
            .collect();
        favorites.sort();
        favorites.dedup();
        let mut recents: Vec<RecentEntry> = self
            .recents
            .iter()
            .map(|e| RecentEntry {
                id: data_format_remap(&e.id).to_owned(),
                used_at: e.used_at,
            })
            .collect();
        // Newest first; keep first occurrence of each id.
        recents.sort_by_key(|a| std::cmp::Reverse(a.used_at));
        let mut seen = std::collections::BTreeSet::new();
        recents.retain(|e| seen.insert(e.id.clone()));
        recents.truncate(MAX_RECENTS);
        Self { favorites, recents }
    }

    pub fn is_favorite(&self, id: &str) -> bool {
        self.favorites.iter().any(|f| f == id)
    }

    pub fn toggle_favorite(&mut self, id: &str) {
        if let Some(pos) = self.favorites.iter().position(|f| f == id) {
            self.favorites.remove(pos);
        } else {
            self.favorites.push(id.to_owned());
            self.favorites.sort();
            self.favorites.dedup();
        }
    }

    pub fn touch_recent(&mut self, id: &str) {
        let now = now_secs();
        self.recents.retain(|e| e.id != id);
        self.recents.insert(
            0,
            RecentEntry {
                id: id.to_owned(),
                used_at: now,
            },
        );
        self.recents.truncate(MAX_RECENTS);
    }

    pub fn recent_ids(&self) -> Vec<&str> {
        self.recents.iter().map(|e| e.id.as_str()).collect()
    }
}

pub fn data_format_remap(id: &str) -> &str {
    devtools_tools::data_format::remap_legacy_id(id)
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Portable data directory for prefs.
///
/// - Linux: `$XDG_DATA_HOME/devtools` or `~/.local/share/devtools`
/// - macOS: `~/Library/Application Support/Devtools`
/// - other: `~/.local/share/devtools` (same as Linux fallback)
pub fn data_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("Devtools");
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(xdg) = std::env::var_os("XDG_DATA_HOME") {
            return PathBuf::from(xdg).join("devtools");
        }
        if let Some(home) = std::env::var_os("HOME") {
            return PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("devtools");
        }
    }
    PathBuf::from(".devtools-data")
}

pub fn prefs_path() -> PathBuf {
    data_dir().join(PREFS_FILE)
}

pub fn load() -> Prefs {
    load_from(&prefs_path()).unwrap_or_default()
}

pub fn load_from(path: &Path) -> Result<Prefs, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Prefs::from_json(&text)
}

pub fn save(prefs: &Prefs) -> Result<(), String> {
    save_to(&prefs_path(), prefs)
}

pub fn save_to(path: &Path, prefs: &Prefs) -> Result<(), String> {
    let json = prefs.to_json()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, json).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn roundtrip_ids_only() {
        let mut prefs = Prefs::default();
        prefs.touch_recent("json");
        prefs.touch_recent("jwt");
        prefs.toggle_favorite("json");
        let text = prefs.to_json().unwrap();
        assert!(text.contains("\"json\""));
        assert!(text.contains("\"jwt\""));
        // No payload-looking keys — only ids + timestamps.
        assert!(!text.contains("input"));
        assert!(!text.contains("output"));
        assert!(!text.contains("token"));
        assert!(!text.contains("\"body\""));
        assert!(!text.contains("\"url\""));
        let loaded = Prefs::from_json(&text).unwrap();
        assert!(loaded.is_favorite("json"));
        assert_eq!(loaded.recent_ids(), vec!["jwt", "json"]);
    }

    #[test]
    fn toggle_favorite_off() {
        let mut prefs = Prefs::default();
        prefs.toggle_favorite("base64");
        assert!(prefs.is_favorite("base64"));
        prefs.toggle_favorite("base64");
        assert!(!prefs.is_favorite("base64"));
    }

    #[test]
    fn rejects_multiline_ids() {
        let bad = r#"{"favorites":["json\nsecret"],"recents":[]}"#;
        assert!(Prefs::from_json(bad).is_err());
    }

    #[test]
    fn remaps_legacy_data_format_ids() {
        let raw = r#"{"favorites":["json-yaml","xml-tool"],"recents":[{"id":"json-toml","used_at":1},{"id":"yaml-toml","used_at":2}]}"#;
        let prefs = Prefs::from_json(raw).unwrap();
        assert!(prefs.is_favorite("data-format"));
        assert!(prefs.is_favorite("xml-tool"));
        assert!(!prefs.is_favorite("json-yaml"));
        assert_eq!(prefs.recent_ids(), vec!["data-format"]);
    }

    #[test]
    fn save_load_temp_file() {
        let dir = std::env::temp_dir().join(format!(
            "devtools-prefs-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = dir.join("prefs.json");
        let mut prefs = Prefs::default();
        prefs.touch_recent("diff");
        prefs.toggle_favorite("diff");
        save_to(&path, &prefs).unwrap();
        let loaded = load_from(&path).unwrap();
        assert_eq!(loaded.recent_ids(), vec!["diff"]);
        assert!(loaded.is_favorite("diff"));
        let raw = fs::read_to_string(&path).unwrap();
        assert!(!raw.contains("payload"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn data_dir_ends_with_devtools() {
        let dir = data_dir();
        let name = dir.file_name().and_then(|s| s.to_str()).unwrap_or("");
        assert!(
            name.eq_ignore_ascii_case("devtools") || name == "Devtools",
            "unexpected data dir {dir:?}"
        );
    }
}
