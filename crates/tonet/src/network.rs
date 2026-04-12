//! Blocking HTTP fetch with Tonet policies: scheme allowlist, dynamic User-Agent, and the
//! **Purity filter** rejecting bodies larger than 1 MiB.

use std::time::Duration;

use anyhow::{anyhow, Context};
use url::Url;

/// Maximum allowed downloaded HTML size (1 MB).
const MAX_BODY_BYTES: usize = 1_000_000;

/// Blocking GET; returns body as UTF-8 text.
///
/// - User-Agent: `Tonet/<version> (Minimalist Browser)`.
/// - Only `http` and `https` schemes.
/// - Only **200 OK** (other status codes return a clear error).
/// - Bodies over 1 MB are rejected (Purity filter).
pub fn fetch_url(url: &str) -> Result<String, anyhow::Error> {
    let parsed = Url::parse(url).with_context(|| format!("Invalid URL: {url}"))?;
    let scheme = parsed.scheme();
    if scheme != "http" && scheme != "https" {
        return Err(anyhow!(
            "Only http and https URLs are allowed (got scheme: {scheme})"
        ));
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!(
            "Tonet/{} (Minimalist Browser)",
            env!("CARGO_PKG_VERSION")
        ))
        .timeout(Duration::from_secs(45))
        .build()
        .context("Could not build HTTP client")?;

    let response = client
        .get(parsed.as_str())
        .send()
        .with_context(|| format!("Request failed for {url}"))?;

    let status = response.status();
    if status != reqwest::StatusCode::OK {
        return Err(anyhow!(
            "HTTP error: server responded with {} (expected 200 OK)",
            status
        ));
    }

    let bytes = response
        .bytes()
        .context("Could not read response body")?;

    if bytes.len() > MAX_BODY_BYTES {
        return Err(anyhow!(
            "Tonet: page too large (over 1 MB). Tonet only loads lightweight content."
        ));
    }

    let text = String::from_utf8(bytes.to_vec())
        .map_err(|e| anyhow!("Body is not valid UTF-8: {e}"))?;

    Ok(text)
}
