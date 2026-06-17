//! Detect an existing per-user Tonet install and its recorded version.

use std::fs;
use std::path::Path;

use crate::version;

#[derive(Debug, Clone)]
struct InstalledVersion {
    version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallAction {
    /// Nothing on disk — perform a fresh install.
    Fresh,
    /// Same or newer than `latest` — do not reinstall.
    AlreadyCurrent { installed_version: String },
    /// Older than `latest` — download and replace in place.
    Update { from_version: String },
}

pub fn resolve_install_action(latest: &str) -> InstallAction {
    plan_install(latest, detect_local_install())
}

fn plan_install(latest: &str, local: Option<InstalledVersion>) -> InstallAction {
    let Some(local) = local else {
        return InstallAction::Fresh;
    };
    let Some(installed) = local.version.filter(|v| !v.trim().is_empty()) else {
        // Legacy install without version metadata — treat as update candidate.
        return InstallAction::Update {
            from_version: "?".into(),
        };
    };
    if version::is_up_to_date(&installed, latest) {
        InstallAction::AlreadyCurrent {
            installed_version: installed,
        }
    } else {
        InstallAction::Update {
            from_version: installed,
        }
    }
}

#[cfg(windows)]
fn detect_local_install() -> Option<InstalledVersion> {
    let install_dir = crate::uninstall::default_install_dir();
    if !install_dir.join("tonet.exe").is_file() {
        return None;
    }
    Some(InstalledVersion {
        version: read_windows_installed_version(&install_dir),
    })
}

#[cfg(windows)]
fn read_windows_installed_version(install_dir: &Path) -> Option<String> {
    crate::uninstall::read_registry_display_version()
        .or_else(|| read_version_file(install_dir))
}

#[cfg(target_os = "linux")]
fn detect_local_install() -> Option<InstalledVersion> {
    if let Some(home) = dirs::home_dir() {
        let install_dir = home.join(".local/share/tonet");
        if install_dir.join("tonet").is_file() {
            return Some(InstalledVersion {
                version: read_version_file(&install_dir),
            });
        }
    }
    if std::path::Path::new("/usr/bin/tonet").is_file() {
        return Some(InstalledVersion {
            version: read_version_file(std::path::Path::new("/usr/share/tonet")),
        });
    }
    None
}

#[cfg(target_os = "macos")]
fn detect_local_install() -> Option<InstalledVersion> {
    let home = dirs::home_dir()?;
    let install_dir = home.join("Applications/Tonet");
    if !install_dir.join("tonet").is_file() {
        return None;
    }
    Some(InstalledVersion {
        version: read_version_file(&install_dir),
    })
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
fn detect_local_install() -> Option<InstalledVersion> {
    None
}

pub fn write_version_file(dir: &Path, version: &str) -> std::io::Result<()> {
    fs::write(dir.join("version.txt"), version)
}

fn read_version_file(dir: &Path) -> Option<String> {
    let raw = fs::read_to_string(dir.join("version.txt")).ok()?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
