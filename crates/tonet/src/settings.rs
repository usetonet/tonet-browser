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

/// Chrome and page palette (persisted).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum UiTheme {
    #[default]
    Dark,
    Light,
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

fn default_ui_scale() -> f32 {
    1.0
}

/// One tile on the New Tab shortcut grid (persisted in [`AppSettings::new_tab_shortcuts`]).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewTabShortcut {
    pub icon: String,
    pub label: String,
    pub url: String,
}

/// Maximum number of URL tiles on the New Tab page (the “Add” control does not count).
pub const NEW_TAB_SHORTCUTS_MAX: usize = 24;

/// Returns true if `url` is allowed as a New Tab shortcut target.
pub fn is_allowed_new_tab_url(url: &str) -> bool {
    let t = url.trim();
    !t.is_empty()
        && (t.starts_with("https://")
            || t.starts_with("http://")
            || t.starts_with("tonet://"))
}

/// Built-in shortcuts when no `new_tab_shortcuts` key exists in settings JSON.
pub fn default_new_tab_shortcuts() -> Vec<NewTabShortcut> {
    vec![
        NewTabShortcut {
            icon: "𝐓".into(),
            label: "Tonet Home".into(),
            url: "https://usetonet.com".into(),
        },
        NewTabShortcut {
            icon: "⊙".into(),
            label: "GitHub".into(),
            url: "https://github.com".into(),
        },
        NewTabShortcut {
            icon: "G".into(),
            label: "Google".into(),
            url: "https://google.com".into(),
        },
        NewTabShortcut {
            icon: "🛡".into(),
            label: "Brave Search".into(),
            url: "https://search.brave.com".into(),
        },
        NewTabShortcut {
            icon: "⚙".into(),
            label: "Tonet settings".into(),
            url: "tonet://settings".into(),
        },
        NewTabShortcut {
            icon: "⤓".into(),
            label: "Downloads".into(),
            url: "tonet://downloads".into(),
        },
        NewTabShortcut {
            icon: "🕐".into(),
            label: "History".into(),
            url: "tonet://history".into(),
        },
        NewTabShortcut {
            icon: "e".into(),
            label: "egui docs".into(),
            url: "https://docs.rs/egui".into(),
        },
        NewTabShortcut {
            icon: "🦀".into(),
            label: "rust-lang.org".into(),
            url: "https://rust-lang.org".into(),
        },
        NewTabShortcut {
            icon: "◉".into(),
            label: "openai.com".into(),
            url: "https://openai.com".into(),
        },
        NewTabShortcut {
            icon: "W".into(),
            label: "wikipedia.org".into(),
            url: "https://wikipedia.org".into(),
        },
    ]
}

/// Settings stored on disk.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppSettings {
    /// `auto` follows the OS locale; or `en`, `es`, `de`, `fr`.
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    #[serde(default)]
    pub ui_theme: UiTheme,
    /// UI zoom multiplier on top of the window’s native scale (0.75–2.0; 1.0 = 100%).
    #[serde(default = "default_ui_scale")]
    pub ui_scale: f32,
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
    /// Preferred download folder for on-disk HTML snapshots; must exist if set.
    #[serde(default)]
    pub download_directory: Option<String>,
    #[serde(default = "default_new_tab_shortcuts")]
    pub new_tab_shortcuts: Vec<NewTabShortcut>,
    #[serde(default)]
    pub system: SystemSettings,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            ui_language: default_ui_language(),
            ui_theme: UiTheme::default(),
            ui_scale: default_ui_scale(),
            search_engine: SearchEngine::default(),
            update_policy: UpdatePolicy::default(),
            last_update_check_unix: None,
            startup_policy: StartupPolicy::default(),
            startup_urls: default_startup_urls(),
            download_directory: None,
            new_tab_shortcuts: default_new_tab_shortcuts(),
            system: SystemSettings::default(),
        }
    }
}

impl AppSettings {
    /// Clamped UI scale for egui (`pixels_per_point` multiplier vs native integration scale).
    #[inline]
    pub fn clamped_ui_scale(&self) -> f32 {
        if self.ui_scale.is_finite() {
            self.ui_scale.clamp(0.75, 2.0)
        } else {
            1.0
        }
    }

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
        let mut s = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str::<AppSettings>(&s).ok())
            .unwrap_or_default();
        if s.new_tab_shortcuts.is_empty() {
            s.new_tab_shortcuts = default_new_tab_shortcuts();
        }
        if !s.ui_scale.is_finite() {
            s.ui_scale = default_ui_scale();
        } else {
            s.ui_scale = s.ui_scale.clamp(0.75, 2.0);
        }
        s
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_new_tab_urls() {
        assert!(is_allowed_new_tab_url("https://a.com"));
        assert!(is_allowed_new_tab_url("http://b.org"));
        assert!(is_allowed_new_tab_url("tonet://settings"));
        assert!(!is_allowed_new_tab_url(""));
        assert!(!is_allowed_new_tab_url("ftp://x"));
    }

    #[test]
    fn ui_scale_clamped() {
        let mut s = AppSettings::default();
        assert_eq!(s.clamped_ui_scale(), 1.0);
        s.ui_scale = 0.5;
        assert_eq!(s.clamped_ui_scale(), 0.75);
        s.ui_scale = 3.0;
        assert_eq!(s.clamped_ui_scale(), 2.0);
        s.ui_scale = f32::NAN;
        assert_eq!(s.clamped_ui_scale(), 1.0);
    }
}
