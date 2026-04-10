//! Version check against GitHub Releases (does not auto-install binaries).

use anyhow::Context;
use semver::Version;
use serde::Deserialize;

const RELEASES_LATEST: &str = "https://api.github.com/repos/usetonet/tonet-browser/releases/latest";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// If GitHub has a release with semver **strictly greater** than this binary, returns that version.
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
        .get(RELEASES_LATEST)
        .send()
        .context("request latest release")?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "GitHub returned {} while checking for updates.",
            resp.status()
        );
    }

    let body: GitHubRelease = resp.json().context("parse GitHub JSON")?;
    let tag = body.tag_name.trim().trim_start_matches('v');
    let remote = Version::parse(tag).with_context(|| format!("invalid remote version: {tag}"))?;

    if remote > current {
        Ok(Some(remote))
    } else {
        Ok(None)
    }
}

/// Opens the downloads page in the system default browser.
pub fn open_downloads_page() {
    let _ = webbrowser::open("https://github.com/usetonet/tonet-browser/releases/latest");
}
