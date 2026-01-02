use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const VALID_SORT_TYPES: &[&str] = &["completed", "uncompleted", "notes", "events"];

fn default_sort_order() -> Vec<String> {
    vec![
        "completed".to_string(),
        "events".to_string(),
        "notes".to_string(),
        "uncompleted".to_string(),
    ]
}

fn default_favorite_tags() -> Vec<String> {
    vec![
        "feature".to_string(),
        "bug".to_string(),
        "idea".to_string(),
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub default_file: Option<String>,
    #[serde(default = "default_sort_order")]
    pub sort_order: Vec<String>,
    #[serde(default = "default_favorite_tags")]
    pub favorite_tags: Vec<String>,
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

    /// Get favorite tag by number key (1-9 maps to index 0-8, 0 maps to index 9)
    #[must_use]
    pub fn get_favorite_tag(&self, key: char) -> Option<&str> {
        let index = match key {
            '1'..='9' => (key as usize) - ('1' as usize),
            '0' => 9,
            _ => return None,
        };
        self.favorite_tags
            .get(index)
            .map(String::as_str)
            .filter(|s| !s.is_empty())
    }

    pub fn load() -> io::Result<Self> {
        let path = get_config_path();
        if path.exists() {
            let content = fs::read_to_string(&path)?;
            toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
        } else {
            Ok(Config::default())
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

        fs::write(&path, include_str!("config_template.toml"))?;
        Ok(true)
    }

    pub fn get_journal_path(&self) -> PathBuf {
        if let Some(ref file) = self.default_file {
            let path = PathBuf::from(file);
            if path.is_absolute() {
                path
            } else {
                std::env::current_dir().unwrap_or_default().join(path)
            }
        } else {
            get_default_journal_path()
        }
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
    get_config_dir().join("journals").join("journal.md")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_favorite_tag() {
        let config = Config {
            favorite_tags: vec!["work".to_string(), "personal".to_string()],
            ..Default::default()
        };

        assert_eq!(config.get_favorite_tag('1'), Some("work"));
        assert_eq!(config.get_favorite_tag('2'), Some("personal"));
        assert_eq!(config.get_favorite_tag('3'), None);
        assert_eq!(config.get_favorite_tag('0'), None);
    }

    #[test]
    fn test_get_favorite_tag_empty_string() {
        let config = Config {
            favorite_tags: vec!["".to_string()],
            ..Default::default()
        };

        assert_eq!(config.get_favorite_tag('1'), None);
    }

    #[test]
    fn test_get_favorite_tag_tenth_slot() {
        let mut tags = vec!["".to_string(); 9];
        tags.push("tenth".to_string());

        let config = Config {
            favorite_tags: tags,
            ..Default::default()
        };

        assert_eq!(config.get_favorite_tag('0'), Some("tenth"));
    }
}
