//! Resolve download URLs from GitHub Releases, with CDN manifest fallback.

use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

const DEFAULT_OWNER: &str = "usetonet";
const DEFAULT_REPO: &str = "tonet-browser";
const DEFAULT_CDN_BASE: &str = "https://dl.usetonet.com";

#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("No suitable download URL for this platform")]
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
}

#[derive(Debug, Clone)]
pub struct PlatformUrls {
    pub version: String,
    pub windows_portable_zip: Option<String>,
    pub linux_deb: Option<String>,
    pub macos_tgz: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CdnManifest {
    version: String,
    #[serde(default)]
    channels: Option<CdnChannels>,
    #[serde(default)]
    download: Option<CdnDownloadBlock>,
}

#[derive(Debug, Deserialize)]
struct CdnChannels {
    #[serde(default)]
    stable: Option<CdnChannel>,
}

#[derive(Debug, Deserialize)]
struct CdnChannel {
    version: String,
    download: CdnDownloadBlock,
}

#[derive(Debug, Deserialize)]
struct CdnDownloadBlock {
    #[serde(rename = "windowsPortableZip")]
    windows_portable_zip: Option<String>,
    deb: Option<String>,
    #[serde(rename = "macTarball")]
    mac_tarball: Option<String>,
}

fn strip_version_tag(tag: &str) -> String {
    tag.trim_start_matches('v').to_string()
}

fn http_client() -> Result<reqwest::blocking::Client, ReleaseError> {
    reqwest::blocking::Client::builder()
        .user_agent("TonetSetup/2.0 (https://usetonet.com)")
        .build()
        .map_err(|e| ReleaseError::Http(e.to_string()))
}

pub fn fetch_latest_release() -> Result<GhRelease, ReleaseError> {
    let owner = std::env::var("TONET_RELEASE_OWNER").unwrap_or_else(|_| DEFAULT_OWNER.into());
    let repo = std::env::var("TONET_RELEASE_REPO").unwrap_or_else(|_| DEFAULT_REPO.into());
    let url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    let client = http_client()?;
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

pub fn resolve_assets_from_github(release: &GhRelease) -> Result<PlatformUrls, ReleaseError> {
    let version = strip_version_tag(&release.tag_name);
    let msi_re = Regex::new(r"^Tonet-[0-9]+\.[0-9]+\.[0-9]+-x64\.msi$").expect("regex");
    let zip_re =
        Regex::new(r"^Tonet-[0-9]+\.[0-9]+\.[0-9]+-windows-x64-portable\.zip$").expect("regex");
    let deb_re = Regex::new(r"^tonet_[0-9]+\.[0-9]+\.[0-9]+_amd64\.deb$").expect("regex");
    let mac_re = Regex::new(r"^Tonet-[0-9]+\.[0-9]+\.[0-9]+-macos\.tar\.gz$").expect("regex");

    let mut windows_portable_zip = None;
    let mut linux_deb = None;
    let mut macos_tgz = None;

    for a in &release.assets {
        if zip_re.is_match(&a.name) {
            windows_portable_zip = Some(a.browser_download_url.clone());
        } else if deb_re.is_match(&a.name) {
            linux_deb = Some(a.browser_download_url.clone());
        } else if mac_re.is_match(&a.name) {
            macos_tgz = Some(a.browser_download_url.clone());
        } else if msi_re.is_match(&a.name) {
            // MSI is for offline Inno flow only.
        }
    }

    if windows_portable_zip.is_none() && linux_deb.is_none() && macos_tgz.is_none() {
        return Err(ReleaseError::NoAsset);
    }

    Ok(PlatformUrls {
        version,
        windows_portable_zip,
        linux_deb,
        macos_tgz,
    })
}

fn cdn_base_url() -> String {
    std::env::var("TONET_CDN_BASE_URL").unwrap_or_else(|_| DEFAULT_CDN_BASE.into())
}

pub fn fetch_cdn_urls() -> Result<PlatformUrls, ReleaseError> {
    let base = cdn_base_url();
    let url = format!("{base}/version.json");
    let client = http_client()?;
    let resp = client
        .get(&url)
        .send()
        .map_err(|e| ReleaseError::Http(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ReleaseError::Http(format!(
            "CDN manifest {} {}",
            resp.status(),
            resp.text().unwrap_or_default()
        )));
    }
    let body = resp
        .text()
        .map_err(|e| ReleaseError::Http(e.to_string()))?;
    let manifest: CdnManifest = serde_json::from_str(&body)?;

    let (version, block) = if let Some(stable) = manifest.channels.and_then(|c| c.stable) {
        (stable.version, stable.download)
    } else if let Some(dl) = manifest.download {
        (manifest.version, dl)
    } else {
        return Err(ReleaseError::NoAsset);
    };

    let mut urls = PlatformUrls {
        version,
        windows_portable_zip: block.windows_portable_zip,
        linux_deb: block.deb,
        macos_tgz: block.mac_tarball,
    };

    // Last-resort stable short filenames if manifest omits nested fields.
    if urls.windows_portable_zip.is_none() {
        urls.windows_portable_zip = Some(format!("{base}/Tonet-windows-x64-portable.zip"));
    }
    if urls.linux_deb.is_none() {
        urls.linux_deb = Some(format!("{base}/tonet_amd64.deb"));
    }
    if urls.macos_tgz.is_none() {
        urls.macos_tgz = Some(format!("{base}/Tonet-macos.tar.gz"));
    }

    Ok(urls)
}

/// GitHub first; if it fails, use `https://dl.usetonet.com/version.json`.
pub fn resolve_download_urls() -> Result<PlatformUrls, ReleaseError> {
    match fetch_latest_release().and_then(|r| resolve_assets_from_github(&r)) {
        Ok(urls) => Ok(urls),
        Err(gh_err) => fetch_cdn_urls().map_err(|cdn_err| {
            ReleaseError::Http(format!("GitHub: {gh_err}; CDN: {cdn_err}"))
        }),
    }
}
