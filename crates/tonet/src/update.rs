//! Comprobación de versiones frente a GitHub Releases (sin auto-instalar binarios).

use anyhow::Context;
use semver::Version;
use serde::Deserialize;

const RELEASES_LATEST: &str = "https://api.github.com/repos/usetonet/tonet-browser/releases/latest";

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

/// Si hay una release en GitHub con semver **estrictamente mayor** que la del binario, devuelve esa versión.
pub fn check_for_newer_release() -> anyhow::Result<Option<Version>> {
    let current = Version::parse(env!("CARGO_PKG_VERSION")).context("versión local inválida")?;

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!(
            "Tonet/{} (UpdateCheck; +https://usetonet.com)",
            env!("CARGO_PKG_VERSION")
        ))
        .build()
        .context("cliente HTTP")?;

    let resp = client
        .get(RELEASES_LATEST)
        .send()
        .context("solicitar última release")?;

    if !resp.status().is_success() {
        anyhow::bail!(
            "GitHub respondió {} al consultar actualizaciones.",
            resp.status()
        );
    }

    let body: GitHubRelease = resp.json().context("parsear JSON de GitHub")?;
    let tag = body.tag_name.trim().trim_start_matches('v');
    let remote = Version::parse(tag).with_context(|| format!("versión remota inválida: {tag}"))?;

    if remote > current {
        Ok(Some(remote))
    } else {
        Ok(None)
    }
}

/// Abre la página de descargas en el navegador predeterminado del SO.
pub fn open_downloads_page() {
    let _ = webbrowser::open("https://github.com/usetonet/tonet-browser/releases/latest");
}
