//! Version check against a CDN-hosted manifest (does not auto-install binaries).

use anyhow::Context;
use semver::Version;
use serde::Deserialize;

fn update_manifest_url() -> &'static str {
    match option_env!("TONET_UPDATE_MANIFEST_URL") {
        Some(v) => v,
        None => "https://dl.usetonet.com/version.json",
    }
}

fn downloads_page_url() -> &'static str {
    match option_env!("TONET_DOWNLOADS_PAGE_URL") {
        Some(v) => v,
        None => "https://dl.usetonet.com/",
    }
}

#[derive(Debug, Deserialize)]
struct ManifestRelease {
    version: String,
}

/// If the manifest reports a semver **strictly greater** than this binary, returns that version.
pub fn check_for_newer_release() -> anyhow::Result<Option<Version>> {
    let current = Version::parse(env!("CARGO_PKG_VERSION")).context("invalid local version")?;

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!(
            "Tonet/{} (UpdateCheck; +https://usetonet.com)",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .context("HTTP client")?;

    let resp = client
        .get(update_manifest_url())
        .send()
        .context("request update manifest")?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "Update manifest endpoint returned {} while checking for updates.",
            resp.status()
        );
    }

    let body: ManifestRelease = resp.json().context("parse update manifest JSON")?;
    let remote_text = body.version.trim().trim_start_matches('v');
    let remote = Version::parse(remote_text)
        .with_context(|| format!("invalid remote version: {remote_text}"))?;

    if remote > current {
        Ok(Some(remote))
    } else {
        Ok(None)
    }
}

/// Opens the downloads page in the system default browser.
pub fn open_downloads_page() {
    let _ = webbrowser::open(downloads_page_url());
}
