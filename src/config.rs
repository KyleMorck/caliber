use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::storage::find_git_root;

const VALID_SORT_TYPES: &[&str] = &["completed", "uncompleted", "notes", "events"];

fn default_sort_order() -> Vec<String> {
    vec![
        "completed".to_string(),
        "events".to_string(),
        "notes".to_string(),
        "uncompleted".to_string(),
    ]
}

fn default_favorite_tags() -> HashMap<String, String> {
    let mut m = HashMap::new();
    m.insert("1".to_string(), "feature".to_string());
    m.insert("2".to_string(), "bug".to_string());
    m.insert("3".to_string(), "idea".to_string());
    m
}

fn default_default_filter() -> String {
    "!tasks".to_string()
}

fn default_header_date_format() -> String {
    "%m/%d/%y".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub global_file: Option<String>,
    #[serde(default = "default_sort_order")]
    pub sort_order: Vec<String>,
    #[serde(default = "default_favorite_tags")]
    pub favorite_tags: HashMap<String, String>,
    #[serde(default)]
    pub filters: HashMap<String, String>,
    #[serde(default = "default_default_filter")]
    pub default_filter: String,
    #[serde(default = "default_header_date_format")]
    pub header_date_format: String,
    #[serde(default)]
    pub hide_completed: bool,
}

/// Raw config for deserialization - all fields are Option to distinguish "not set" from "set to default"
#[derive(Debug, Clone, Deserialize, Default)]
struct RawConfig {
    pub global_file: Option<String>,
    pub sort_order: Option<Vec<String>>,
    pub favorite_tags: Option<HashMap<String, String>>,
    pub filters: Option<HashMap<String, String>>,
    pub default_filter: Option<String>,
    pub header_date_format: Option<String>,
    pub hide_completed: Option<bool>,
}

impl RawConfig {
    fn into_config(self) -> Config {
        Config {
            global_file: self.global_file,
            sort_order: self.sort_order.unwrap_or_else(default_sort_order),
            favorite_tags: self.favorite_tags.unwrap_or_else(default_favorite_tags),
            filters: self.filters.unwrap_or_default(),
            default_filter: self.default_filter.unwrap_or_else(default_default_filter),
            header_date_format: self
                .header_date_format
                .unwrap_or_else(default_header_date_format),
            hide_completed: self.hide_completed.unwrap_or(false),
        }
    }

    fn merge_over(self, global: RawConfig) -> RawConfig {
        RawConfig {
            // global_file is NEVER overridden from project
            global_file: global.global_file,
            // Scalars: use project if set, otherwise global
            sort_order: self.sort_order.or(global.sort_order),
            default_filter: self.default_filter.or(global.default_filter),
            header_date_format: self.header_date_format.or(global.header_date_format),
            hide_completed: self.hide_completed.or(global.hide_completed),
            // HashMaps: MERGE (project values override matching global keys)
            favorite_tags: Some(merge_hashmaps(global.favorite_tags, self.favorite_tags)),
            filters: Some(merge_hashmaps(global.filters, self.filters)),
        }
    }
}

fn merge_hashmaps(
    base: Option<HashMap<String, String>>,
    overlay: Option<HashMap<String, String>>,
) -> HashMap<String, String> {
    match (base, overlay) {
        (Some(mut b), Some(o)) => {
            b.extend(o);
            b
        }
        (Some(b), None) => b,
        (None, Some(o)) => o,
        (None, None) => HashMap::new(),
    }
}

impl Config {
    #[must_use]
    pub fn validated_sort_order(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        let result: Vec<String> = self
            .sort_order
            .iter()
            .filter(|s| VALID_SORT_TYPES.contains(&s.as_str()) && seen.insert(s.as_str()))
            .cloned()
            .collect();

        if result.is_empty() {
            default_sort_order()
        } else {
            result
        }
    }

    /// Get favorite tag by number key (0-9)
    #[must_use]
    pub fn get_favorite_tag(&self, key: char) -> Option<&str> {
        if !key.is_ascii_digit() {
            return None;
        }
        self.favorite_tags
            .get(&key.to_string())
            .map(String::as_str)
            .filter(|s| !s.is_empty())
    }

    /// Load global config only (no project merge)
    pub fn load_global() -> io::Result<Self> {
        let global = load_raw_config(get_config_path())?;
        Ok(global.into_config())
    }

    /// Load config, merging project config over global config if present
    pub fn load_merged() -> io::Result<Self> {
        let global = load_raw_config(get_config_path())?;

        if let Some(project) = load_project_config() {
            Ok(project.merge_over(global).into_config())
        } else {
            Ok(global.into_config())
        }
    }

    pub fn init() -> io::Result<bool> {
        let path = get_config_path();
        if path.exists() {
            return Ok(false);
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&path, "")?;
        Ok(true)
    }

    pub fn get_global_journal_path(&self) -> PathBuf {
        if let Some(ref file) = self.global_file {
            resolve_path(file)
        } else {
            get_default_journal_path()
        }
    }
}

/// Resolve a path to absolute, joining with cwd if relative.
#[must_use]
pub fn resolve_path(path: &str) -> PathBuf {
    let path = PathBuf::from(path);
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir().unwrap_or_default().join(path)
    }
}

pub fn get_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("caliber")
}

pub fn get_config_path() -> PathBuf {
    get_config_dir().join("config.toml")
}

pub fn get_default_journal_path() -> PathBuf {
    get_config_dir().join("global_journal.md")
}

fn load_raw_config(path: PathBuf) -> io::Result<RawConfig> {
    if path.exists() {
        let content = fs::read_to_string(&path)?;
        toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    } else {
        Ok(RawConfig::default())
    }
}

fn load_project_config() -> Option<RawConfig> {
    let root = find_git_root()?;
    let path = root.join(".caliber").join("config.toml");
    if path.exists() {
        let content = fs::read_to_string(&path).ok()?;
        toml::from_str(&content).ok()
    } else {
        None
    }
}
