//! Persistent Tonet preferences (JSON in the OS config directory).

use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

/// When to check GitHub Releases for updates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePolicy {
    #[default]
    OnStartup,
    Periodic,
    ManualOnly,
}

/// Default web search used when omnibox input is not a URL (omnibox + New Tab search).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SearchEngine {
    #[default]
    Duckduckgo,
    Google,
    Brave,
}

fn default_ui_language() -> String {
    "auto".to_string()
}

/// Settings stored on disk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    /// `auto` follows the OS locale; or `en`, `es`, `de`, `fr`.
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    #[serde(default)]
    pub search_engine: SearchEngine,
    pub update_policy: UpdatePolicy,
    #[serde(default)]
    pub last_update_check_unix: Option<i64>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ui_language: default_ui_language(),
            search_engine: SearchEngine::default(),
            update_policy: UpdatePolicy::default(),
            last_update_check_unix: None,
        }
    }
}

impl AppSettings {
    pub fn file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("tonet").join("settings.json"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::file_path() else {
            return Self::default();
        };
        if !path.exists() {
            return Self::default();
        }
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let Some(path) = Self::file_path() else {
            anyhow::bail!("No config directory on this system.");
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("mkdir {:?}", parent))?;
        }
        let json = serde_json::to_string_pretty(self).context("serialize settings")?;
        fs::write(&path, json).with_context(|| format!("write {:?}", path))?;
        Ok(())
    }
}
