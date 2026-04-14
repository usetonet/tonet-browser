//! Resolve latest Tonet release assets from GitHub.

use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_OWNER: &str = "usetonet";
const DEFAULT_REPO: &str = "tonet-browser";

#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("No suitable asset found for this platform in the latest release")]
    NoAsset,
}

#[derive(Debug, Deserialize)]
pub struct GhRelease {
    pub tag_name: String,
    pub assets: Vec<GhAsset>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GhAsset {
    pub name: String,
    pub browser_download_url: String,
    #[allow(dead_code)]
    pub size: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ResolvedAssets {
    pub tag: String,
    /// Parsed semver from `tag_name` (without leading `v`).
    pub version: String,
    /// Windows MSI (per-machine installer).
    pub windows_msi: Option<GhAsset>,
    /// Windows portable zip (single `tonet.exe`).
    pub windows_portable_zip: Option<GhAsset>,
    /// Linux .deb package.
    pub linux_deb: Option<GhAsset>,
    /// macOS tarball with `tonet` binary.
    pub macos_tgz: Option<GhAsset>,
}

fn strip_version_tag(tag: &str) -> String {
    tag.trim_start_matches('v').to_string()
}

pub fn fetch_latest_release() -> Result<GhRelease, ReleaseError> {
    let owner = std::env::var("TONET_RELEASE_OWNER").unwrap_or_else(|_| DEFAULT_OWNER.into());
    let repo = std::env::var("TONET_RELEASE_REPO").unwrap_or_else(|_| DEFAULT_REPO.into());
    let url = format!(
        "https://api.github.com/repos/{owner}/{repo}/releases/latest"
    );
    let client = reqwest::blocking::Client::builder()
        .user_agent("TonetSetup/1.0 (https://usetonet.com)")
        .build()
        .map_err(|e| ReleaseError::Http(e.to_string()))?;

    let resp = client
        .get(&url)
        .send()
        .map_err(|e| ReleaseError::Http(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ReleaseError::Http(format!(
            "{} {}",
            resp.status(),
            resp.text().unwrap_or_default()
        )));
    }
    let body = resp
        .text()
        .map_err(|e| ReleaseError::Http(e.to_string()))?;
    Ok(serde_json::from_str(&body)?)
}

pub fn resolve_assets(release: &GhRelease) -> Result<ResolvedAssets, ReleaseError> {
    let version = strip_version_tag(&release.tag_name);
    let msi_re = Regex::new(r"^Tonet-[0-9]+\.[0-9]+\.[0-9]+-x64\.msi$").expect("regex");
    let zip_re =
        Regex::new(r"^Tonet-[0-9]+\.[0-9]+\.[0-9]+-windows-x64-portable\.zip$").expect("regex");
    let deb_re = Regex::new(r"^tonet_[0-9]+\.[0-9]+\.[0-9]+_amd64\.deb$").expect("regex");
    let mac_re = Regex::new(r"^Tonet-[0-9]+\.[0-9]+\.[0-9]+-macos\.tar\.gz$").expect("regex");

    let mut windows_msi = None;
    let mut windows_portable_zip = None;
    let mut linux_deb = None;
    let mut macos_tgz = None;

    for a in &release.assets {
        if msi_re.is_match(&a.name) {
            windows_msi = Some(a.clone());
        } else if zip_re.is_match(&a.name) {
            windows_portable_zip = Some(a.clone());
        } else if deb_re.is_match(&a.name) {
            linux_deb = Some(a.clone());
        } else if mac_re.is_match(&a.name) {
            macos_tgz = Some(a.clone());
        }
    }

    let has_any = windows_msi.is_some()
        || windows_portable_zip.is_some()
        || linux_deb.is_some()
        || macos_tgz.is_some();
    if !has_any {
        return Err(ReleaseError::NoAsset);
    }

    Ok(ResolvedAssets {
        tag: release.tag_name.clone(),
        version,
        windows_msi,
        windows_portable_zip,
        linux_deb,
        macos_tgz,
    })
}
