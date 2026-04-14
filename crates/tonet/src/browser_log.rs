//! Persistent visit and download-style activity log (JSON under the Tonet config dir).

use std::fs;
use std::path::PathBuf;

use anyhow::Context;
use serde::{Deserialize, Serialize};

const MAX_VISITS: usize = 5_000;
const MAX_DOWNLOADS: usize = 2_000;

fn tonet_config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("tonet"))
}

pub fn visit_history_path() -> Option<PathBuf> {
    tonet_config_dir().map(|d| d.join("visit_history.json"))
}

pub fn download_history_path() -> Option<PathBuf> {
    tonet_config_dir().map(|d| d.join("download_history.json"))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VisitRecord {
    pub id: u64,
    pub url: String,
    pub title: Option<String>,
    pub visited_at_unix: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadRecord {
    pub id: u64,
    pub url: String,
    pub display_name: String,
    pub finished_at_unix: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct VisitFile {
    pub next_id: u64,
    pub visits: Vec<VisitRecord>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
struct DownloadFile {
    pub next_id: u64,
    pub items: Vec<DownloadRecord>,
}

#[derive(Debug)]
pub struct BrowserLog {
    visit_next_id: u64,
    pub visits: Vec<VisitRecord>,
    download_next_id: u64,
    pub downloads: Vec<DownloadRecord>,
}

impl BrowserLog {
    pub fn load() -> Self {
        let visits = Self::load_visits();
        let downloads = Self::load_downloads();
        let visit_next_id = visits.iter().map(|v| v.id).max().unwrap_or(0).saturating_add(1);
        let download_next_id = downloads
            .iter()
            .map(|d| d.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
        Self {
            visit_next_id,
            visits,
            download_next_id,
            downloads,
        }
    }

    fn load_visits() -> Vec<VisitRecord> {
        let Some(path) = visit_history_path() else {
            return Vec::new();
        };
        if !path.exists() {
            return Vec::new();
        }
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<VisitFile>(&s).ok())
            .map(|f| f.visits)
            .unwrap_or_default()
    }

    fn load_downloads() -> Vec<DownloadRecord> {
        let Some(path) = download_history_path() else {
            return Vec::new();
        };
        if !path.exists() {
            return Vec::new();
        }
        fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<DownloadFile>(&s).ok())
            .map(|f| f.items)
            .unwrap_or_default()
    }

    fn write_visits(&self) -> anyhow::Result<()> {
        let Some(path) = visit_history_path() else {
            anyhow::bail!("No config directory.");
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("mkdir {:?}", parent))?;
        }
        let payload = VisitFile {
            next_id: self.visit_next_id,
            visits: self.visits.clone(),
        };
        let json = serde_json::to_string_pretty(&payload).context("serialize visits")?;
        fs::write(&path, json).with_context(|| format!("write {:?}", path))?;
        Ok(())
    }

    fn write_downloads(&self) -> anyhow::Result<()> {
        let Some(path) = download_history_path() else {
            anyhow::bail!("No config directory.");
        };
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| format!("mkdir {:?}", parent))?;
        }
        let payload = DownloadFile {
            next_id: self.download_next_id,
            items: self.downloads.clone(),
        };
        let json = serde_json::to_string_pretty(&payload).context("serialize downloads")?;
        fs::write(&path, json).with_context(|| format!("write {:?}", path))?;
        Ok(())
    }

    pub fn record_visit(&mut self, url: String, title: Option<String>) {
        let now = chrono::Utc::now().timestamp();
        let id = self.visit_next_id;
        self.visit_next_id = self.visit_next_id.saturating_add(1);
        self.visits.push(VisitRecord {
            id,
            url,
            title,
            visited_at_unix: now,
        });
        if self.visits.len() > MAX_VISITS {
            let drop = self.visits.len() - MAX_VISITS;
            self.visits.drain(0..drop);
        }
        let _ = self.write_visits();
    }

    pub fn remove_visits(&mut self, ids: &[u64]) {
        self.visits.retain(|v| !ids.contains(&v.id));
        let _ = self.write_visits();
    }

    pub fn clear_visits(&mut self) {
        self.visits.clear();
        let _ = self.write_visits();
    }

    pub fn record_page_fetch(&mut self, url: &str, title: Option<String>) {
        let now = chrono::Utc::now().timestamp();
        let display_name = title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| display_name_from_url(url));
        let id = self.download_next_id;
        self.download_next_id = self.download_next_id.saturating_add(1);
        self.downloads.push(DownloadRecord {
            id,
            url: url.to_string(),
            display_name,
            finished_at_unix: now,
        });
        if self.downloads.len() > MAX_DOWNLOADS {
            let drop = self.downloads.len() - MAX_DOWNLOADS;
            self.downloads.drain(0..drop);
        }
        let _ = self.write_downloads();
    }

    pub fn remove_downloads(&mut self, ids: &[u64]) {
        self.downloads.retain(|d| !ids.contains(&d.id));
        let _ = self.write_downloads();
    }

    pub fn clear_downloads(&mut self) {
        self.downloads.clear();
        let _ = self.write_downloads();
    }
}

fn display_name_from_url(url: &str) -> String {
    url::Url::parse(url)
        .ok()
        .and_then(|u| {
            u.path_segments()
                .and_then(|mut s| s.next_back().map(|s| s.to_string()))
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| "document".to_string())
}
