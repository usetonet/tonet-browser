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

/// What to do when Tonet starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StartupPolicy {
    #[default]
    NewTabPage,
    RestoreSession,
    OpenSpecificPages,
}

/// Energy saver sub-mode (UI only until a power backend exists).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EnergySaverMode {
    #[default]
    WhenBatteryLow,
    WhenUnplugged,
}

fn default_true() -> bool {
    true
}

/// System-style toggles mirrored from common browser settings pages.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemSettings {
    #[serde(default)]
    pub continue_background_apps: bool,
    #[serde(default = "default_true")]
    pub use_hardware_acceleration: bool,
    #[serde(default = "default_true")]
    pub close_window_when_last_tab: bool,
    #[serde(default = "default_true")]
    pub warn_before_closing_multi_tab_window: bool,
    #[serde(default = "default_true")]
    pub show_fullscreen_esc_reminder: bool,
    #[serde(default)]
    pub vpn_use_wireguard: bool,
    #[serde(default = "default_true")]
    pub vpn_show_tray_icon: bool,
    #[serde(default)]
    pub memory_saver_enabled: bool,
    #[serde(default = "default_true")]
    pub energy_saver_enabled: bool,
    #[serde(default)]
    pub energy_saver_mode: EnergySaverMode,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            continue_background_apps: false,
            use_hardware_acceleration: true,
            close_window_when_last_tab: true,
            warn_before_closing_multi_tab_window: true,
            show_fullscreen_esc_reminder: true,
            vpn_use_wireguard: false,
            vpn_show_tray_icon: true,
            memory_saver_enabled: false,
            energy_saver_enabled: true,
            energy_saver_mode: EnergySaverMode::default(),
        }
    }
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

fn default_startup_urls() -> String {
    String::new()
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
    #[serde(default)]
    pub startup_policy: StartupPolicy,
    /// When [`StartupPolicy::OpenSpecificPages`]: one URL or search per line (same rules as the omnibox).
    #[serde(default = "default_startup_urls")]
    pub startup_urls: String,
    /// Preferred download folder when Tonet saves files to disk (future); must exist if set.
    #[serde(default)]
    pub download_directory: Option<String>,
    #[serde(default)]
    pub system: SystemSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ui_language: default_ui_language(),
            search_engine: SearchEngine::default(),
            update_policy: UpdatePolicy::default(),
            last_update_check_unix: None,
            startup_policy: StartupPolicy::default(),
            startup_urls: default_startup_urls(),
            download_directory: None,
            system: SystemSettings::default(),
        }
    }
}

impl AppSettings {
    /// Effective download directory: valid custom path, else OS Downloads, else `None`.
    pub fn resolved_download_directory(&self) -> Option<PathBuf> {
        if let Some(ref s) = self.download_directory {
            let t = s.trim();
            if !t.is_empty() {
                let p = PathBuf::from(t);
                if p.is_dir() {
                    return Some(p);
                }
            }
        }
        dirs::download_dir()
    }

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
