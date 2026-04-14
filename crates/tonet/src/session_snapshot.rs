//! Last window session (tab URLs + active tab) for "Continue where you left off".

use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

const FILE_NAME: &str = "last_session.json";
const FORMAT_VERSION: u32 = 1;

fn tonet_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("tonet"))
}

pub fn last_session_path() -> Option<PathBuf> {
    tonet_config_dir().map(|d| d.join(FILE_NAME))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub version: u32,
    pub active_tab: usize,
    /// Omnibox URL per tab (empty string = new tab page).
    pub tab_urls: Vec<String>,
}

impl SessionSnapshot {
    pub fn from_app(active_tab: usize, tab_urls: Vec<String>) -> Self {
        Self {
            version: FORMAT_VERSION,
            active_tab,
            tab_urls,
        }
    }

    pub fn load() -> Option<Self> {
        let path = last_session_path()?;
        if !path.exists() {
            return None;
        }
        let s = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&s).ok()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let Some(path) = last_session_path() else {
            anyhow::bail!("No config directory.");
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("mkdir {:?}", parent))?;
        }
        let json = serde_json::to_string_pretty(self).context("serialize session")?;
        fs::write(&path, json).with_context(|| format!("write {:?}", path))?;
        Ok(())
    }

    /// Build tabs; caps count and clamps active tab index.
    pub fn into_tabs(self, max_tabs: usize) -> Option<(Vec<crate::tab::Tab>, usize)> {
        if self.tab_urls.is_empty() {
            return None;
        }
        let mut urls = self.tab_urls;
        if urls.len() > max_tabs {
            urls.truncate(max_tabs);
        }
        let active = self.active_tab.min(urls.len().saturating_sub(1));
        let tabs: Vec<crate::tab::Tab> = urls.into_iter().map(crate::tab::Tab::new).collect();
        Some((tabs, active))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn into_tabs_caps_and_clamps_active() {
        let snap = SessionSnapshot {
            version: 1,
            active_tab: 99,
            tab_urls: vec!["a".into(), "b".into(), "c".into()],
        };
        let (tabs, active) = snap.into_tabs(2).unwrap();
        assert_eq!(tabs.len(), 2);
        assert_eq!(active, 1);
    }

    #[test]
    fn into_tabs_empty_none() {
        let snap = SessionSnapshot {
            version: 1,
            active_tab: 0,
            tab_urls: vec![],
        };
        assert!(snap.into_tabs(10).is_none());
    }
}
