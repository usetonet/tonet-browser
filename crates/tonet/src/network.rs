//! Blocking HTTP fetch with Tonet policies: scheme allowlist, dynamic User-Agent, and the
//! **Purity filter** rejecting bodies larger than 1 MiB.

use std::time::Duration;

use anyhow::{anyhow, Context};
use tonet_engine::document_url;
use tonet_engine::policy;
use tonet_engine::EngineLimits;
use url::Url;

const LIMITS: EngineLimits = EngineLimits::STANDARD;

fn reqwest_redirect_policy() -> reqwest::redirect::Policy {
    let n = LIMITS.max_http_redirects as usize;
    if n == 0 {
        reqwest::redirect::Policy::none()
    } else {
        reqwest::redirect::Policy::limited(n)
    }
}

/// Blocking GET; returns body as UTF-8 text.
///
/// - User-Agent: `Tonet/<version> (Minimalist Browser)`.
/// - Only `http` and `https` schemes.
/// - Follows at most [`EngineLimits::max_http_redirects`] redirects, then expects **200 OK** on the final response (other status codes return a clear error).
/// - Bodies over 1 MB are rejected (Purity filter).
pub fn fetch_url(url: &str) -> Result<String, anyhow::Error> {
    let url = document_url::normalize_document_url_for_http_get(url).map_err(|e| anyhow!(e))?;
    let parsed = Url::parse(&url).with_context(|| format!("Invalid URL: {url}"))?;
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
        .redirect(reqwest_redirect_policy())
        .timeout(Duration::from_secs(LIMITS.http_request_timeout_secs))
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

    policy::check_document_size(bytes.len(), &LIMITS).map_err(|_| {
        anyhow!(
            "Tonet: page too large (over {} bytes). Tonet only loads lightweight content.",
            LIMITS.max_document_bytes
        )
    })?;

    let text = String::from_utf8(bytes.to_vec())
        .map_err(|e| anyhow!("Body is not valid UTF-8: {e}"))?;

    Ok(text)
}

/// Try to fetch a favicon by probing a list of candidate URLs in order.
///
/// Candidates are typically extracted from `<link rel="icon">` tags in the page
/// HTML (via [`tonet_engine::html::minimal::extract_favicon_candidates`]), followed by classic
/// fallback paths like `/favicon.ico`.
pub fn fetch_favicon_from_candidates(candidates: &[String]) -> Option<Vec<u8>> {
    if candidates.is_empty() {
        return None;
    }

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!(
            "Tonet/{} (Minimalist Browser)",
            env!("CARGO_PKG_VERSION")
        ))
        .redirect(reqwest_redirect_policy())
        .timeout(Duration::from_secs(LIMITS.favicon_request_timeout_secs))
        .build()
        .ok()?;

    for url in candidates {
        if url.starts_with("data:") {
            if let Some(bytes) = decode_data_uri(url) {
                if !bytes.is_empty() && bytes.len() <= LIMITS.max_favicon_bytes {
                    return Some(bytes);
                }
            }
            continue;
        }

        let resp = match client.get(url.as_str()).send() {
            Ok(r) if r.status().is_success() => r,
            _ => continue,
        };
        if let Ok(bytes) = resp.bytes() {
            if bytes.is_empty() || bytes.len() > LIMITS.max_favicon_bytes {
                continue;
            }
            if looks_like_html(&bytes) {
                continue;
            }
            return Some(bytes.to_vec());
        }
    }

    None
}

fn decode_data_uri(uri: &str) -> Option<Vec<u8>> {
    let rest = uri.strip_prefix("data:")?;
    if let Some(b64_start) = rest.find(";base64,") {
        let encoded = &rest[b64_start + 8..];
        let clean: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
        base64_decode_bytes(&clean)
    } else {
        None
    }
}

fn base64_decode_bytes(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let lut = |c: u8| -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            b'=' => None,
            _ => None,
        }
    };
    let bytes: Vec<u8> = input.bytes().filter(|&b| b != b'=').collect();
    let chunks = bytes.chunks(4);
    for chunk in chunks {
        let vals: Vec<u8> = chunk.iter().filter_map(|&b| lut(b)).collect();
        if vals.is_empty() {
            break;
        }
        if vals.len() >= 2 {
            out.push((vals[0] << 2) | (vals[1] >> 4));
        }
        if vals.len() >= 3 {
            out.push((vals[1] << 4) | (vals[2] >> 2));
        }
        if vals.len() >= 4 {
            out.push((vals[2] << 6) | vals[3]);
        }
    }
    Some(out)
}

fn looks_like_html(bytes: &[u8]) -> bool {
    let prefix: Vec<u8> = bytes.iter().take(64).copied().collect();
    let lower: Vec<u8> = prefix.iter().map(|b| b.to_ascii_lowercase()).collect();
    lower.starts_with(b"<!doctype")
        || lower.starts_with(b"<html")
        || lower.starts_with(b"<head")
}

/// Guess image file extension from magic bytes.
pub fn guess_favicon_ext(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"\x89PNG") {
        ".png"
    } else if bytes.starts_with(b"\x00\x00\x01\x00") || bytes.starts_with(b"\x00\x00\x02\x00") {
        ".ico"
    } else if bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
        ".svg"
    } else if bytes.starts_with(b"GIF8") {
        ".gif"
    } else if bytes.starts_with(b"\xFF\xD8\xFF") {
        ".jpg"
    } else if bytes.len() > 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP" {
        ".webp"
    } else {
        ".ico"
    }
}
