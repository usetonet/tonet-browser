//! Blocking download + native save dialog for Servo [`load_web_resource`] interception.

use std::fs;
use std::io::{Read, Write};
use std::time::Duration;

use anyhow::{anyhow, Context};
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_TYPE};
use url::Url;

const MAX_DOWNLOAD_BYTES: u64 = 512 * 1024 * 1024;
const DOWNLOAD_TIMEOUT_SECS: u64 = 900;

pub(crate) struct CompletedBackgroundDownload {
    pub source_url: String,
    pub display_name: Option<String>,
    pub saved_path: String,
}

fn reqwest_client() -> anyhow::Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent(format!(
            "Tonet/{} (Minimalist Browser)",
            env!("CARGO_PKG_VERSION")
        ))
        .redirect(reqwest::redirect::Policy::limited(32))
        .timeout(Duration::from_secs(DOWNLOAD_TIMEOUT_SECS))
        .build()
        .context("build HTTP client")
}

fn head_probe_client() -> anyhow::Result<reqwest::blocking::Client> {
    reqwest::blocking::Client::builder()
        .user_agent(format!(
            "Tonet/{} (Minimalist Browser)",
            env!("CARGO_PKG_VERSION")
        ))
        .redirect(reqwest::redirect::Policy::limited(8))
        .timeout(Duration::from_secs(4))
        .build()
        .context("build HEAD probe client")
}

#[inline]
fn mime_suggests_allowlisted_binary(mime_lower: &str) -> bool {
    matches!(
        mime_lower,
        "application/pdf"
            | "application/zip"
            | "application/x-zip-compressed"
            | "application/gzip"
            | "application/x-gzip"
            | "application/x-tar"
            | "application/gtar"
            | "application/x-gtar"
            | "application/vnd.debian.binary-package"
            | "application/vnd.rar"
            | "application/x-rar-compressed"
            | "application/msword"
            | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            | "application/vnd.ms-excel"
            | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
            | "application/wasm"
    )
}

/// Short **`HEAD`** to detect attachment-style responses when the URL has no file extension
/// (used only after [`super::download_heuristic::should_head_probe_main_frame_binary_get`]).
/// **Blocking** on the Servo embedder thread (bounded timeout).
pub(crate) fn head_suggests_intercept_binary_get(url: &str) -> bool {
    let Ok(client) = head_probe_client() else {
        return false;
    };
    let Ok(resp) = client.head(url).send() else {
        return false;
    };
    if !resp.status().is_success() {
        return false;
    }
    if let Some(cd) = resp
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(raw_name) = super::content_disposition::parse_filename_value(cd) {
            if let Some((_, ext)) = raw_name.rsplit_once('.') {
                if !ext.is_empty()
                    && super::download_heuristic::is_allowlisted_download_extension(ext)
                {
                    return true;
                }
            }
        }
    }
    let Some(ct) = resp
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
    else {
        return false;
    };
    let mime = ct.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    mime_suggests_allowlisted_binary(&mime)
}

/// DOS device **stem** (case-insensitive): `CON`, `PRN`, …, `COM1`–`COM9`, `LPT1`–`LPT9`, `CLOCK$`.
#[inline]
fn win32_reserved_device_stem(stem: &str) -> bool {
    let stem = stem.to_ascii_lowercase();
    match stem.as_str() {
        "con" | "prn" | "aux" | "nul" | "clock$" => true,
        _ if stem.len() == 4 && stem.starts_with("com") => stem[3..]
            .parse::<u8>()
            .ok()
            .map_or(false, |n| (1..=9).contains(&n)),
        _ if stem.len() == 4 && stem.starts_with("lpt") => stem[3..]
            .parse::<u8>()
            .ok()
            .map_or(false, |n| (1..=9).contains(&n)),
        _ => false,
    }
}

/// Maps forbidden / control characters to `_`, caps length, then trims ends. Strips trailing **`.`**
/// and **` `** (Win32 `CreateFile` rejects those tails on save-as). Prefixes **`_`** when the name
/// stem is a reserved DOS device (`COM1`, `NUL`, …).
fn sanitize_filename(name: &str) -> String {
    const FORBIDDEN: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut s: String = name
        .chars()
        .map(|c| {
            if FORBIDDEN.contains(&c) || c.is_control() {
                '_'
            } else {
                c
            }
        })
        .take(200)
        .collect::<String>()
        .trim()
        .to_string();
    while let Some(last) = s.chars().last() {
        if last == '.' || last == ' ' {
            s.pop();
        } else {
            break;
        }
    }
    if !s.is_empty() {
        let stem = match s.rsplit_once('.') {
            Some((a, ext)) if !a.is_empty() && !ext.is_empty() => a,
            _ => s.as_str(),
        };
        if win32_reserved_device_stem(stem) {
            s.insert(0, '_');
        }
    }
    s
}

/// Default save-as filename: parsed `Content-Disposition` when present and non-empty after
/// sanitization, otherwise the last non-empty URL path segment, else `"download"`.
fn suggested_save_name_for_download(content_disposition: Option<&str>, source_url: &str) -> String {
    if let Some(cd) = content_disposition {
        if let Some(raw) = super::content_disposition::parse_filename_value(cd) {
            let s = sanitize_filename(&raw);
            if !s.is_empty() {
                return s;
            }
        }
    }
    Url::parse(source_url)
        .ok()
        .and_then(|u| {
            super::url_path::last_non_empty_path_segment(&u).map(|s| s.to_string())
        })
        .map(|s| sanitize_filename(&s))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "download".to_string())
}

fn suggested_save_name(response: &reqwest::blocking::Response, source_url: &str) -> String {
    let cd = response
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok());
    suggested_save_name_for_download(cd, source_url)
}

/// GET `url`, prompt save-as, stream body to disk (bounded size). Does **not** share Servo cookies.
pub(crate) fn download_url_to_user_picked_file(url: &str) -> anyhow::Result<CompletedBackgroundDownload> {
    let parsed = Url::parse(url).with_context(|| format!("invalid URL: {url}"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        anyhow::bail!("only http(s) downloads supported");
    }

    let client = reqwest_client()?;
    let mut response = client
        .get(url)
        .send()
        .with_context(|| format!("GET {url}"))?
        .error_for_status()
        .with_context(|| format!("HTTP error for {url}"))?;

    let suggested = suggested_save_name(&response, url);
    let picked = rfd::FileDialog::new()
        .set_file_name(&suggested)
        .save_file()
        .ok_or_else(|| anyhow!("save dialog cancelled"))?;

    let mut f = fs::File::create(&picked).with_context(|| format!("create {:?}", picked))?;
    let mut written: u64 = 0;
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = response.read(&mut buf).context("read body")?;
        if n == 0 {
            break;
        }
        written = written.saturating_add(n as u64);
        if written > MAX_DOWNLOAD_BYTES {
            anyhow::bail!("download larger than {} bytes", MAX_DOWNLOAD_BYTES);
        }
        f.write_all(&buf[..n]).context("write file")?;
    }

    let display_name = picked
        .file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());

    Ok(CompletedBackgroundDownload {
        source_url: url.to_string(),
        display_name,
        saved_path: picked.to_string_lossy().into_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::{head_suggests_intercept_binary_get, sanitize_filename, suggested_save_name_for_download};

    #[test]
    fn head_suggests_returns_false_for_unparseable_url() {
        assert!(!head_suggests_intercept_binary_get(":::not-a-url:::"));
    }

    #[test]
    fn mime_allowlists_common_binary_types() {
        assert!(super::mime_suggests_allowlisted_binary("application/pdf"));
        assert!(super::mime_suggests_allowlisted_binary("application/zip"));
        assert!(super::mime_suggests_allowlisted_binary(
            "application/vnd.debian.binary-package"
        ));
        assert!(super::mime_suggests_allowlisted_binary("application/wasm"));
    }

    #[test]
    fn mime_rejects_octet_stream_and_html() {
        assert!(!super::mime_suggests_allowlisted_binary("application/octet-stream"));
        assert!(!super::mime_suggests_allowlisted_binary("text/html"));
    }

    #[test]
    fn sanitize_replaces_forbidden_chars() {
        assert_eq!(
            sanitize_filename(r#"bad:name*?"<>|x.zip"#),
            "bad_name______x.zip"
        );
    }

    #[test]
    fn sanitize_replaces_slash() {
        assert_eq!(sanitize_filename("a/b\\c.txt"), "a_b_c.txt");
    }

    #[test]
    fn sanitize_caps_length_before_trim() {
        let long = "x".repeat(250);
        let out = sanitize_filename(&long);
        assert_eq!(out.len(), 200);
        assert!(out.chars().all(|c| c == 'x'));
    }

    #[test]
    fn sanitize_leading_spaces_count_toward_cap() {
        // `take(200)` applies before `trim`, so two leading spaces leave 198 payload bytes.
        let long = "x".repeat(250);
        let out = sanitize_filename(&format!("  {long}"));
        assert_eq!(out.len(), 198);
        assert!(out.chars().all(|c| c == 'x'));
    }

    #[test]
    fn sanitize_maps_control_chars() {
        assert_eq!(
            sanitize_filename("a\u{1}b\n.pdf"),
            "a_b_.pdf"
        );
    }

    #[test]
    fn sanitize_strips_trailing_dots_and_spaces_win32() {
        assert_eq!(sanitize_filename("report.pdf. "), "report.pdf");
        assert_eq!(sanitize_filename("name..."), "name");
        assert_eq!(sanitize_filename(". . "), "");
    }

    #[test]
    fn sanitize_prefixes_reserved_device_names() {
        assert_eq!(sanitize_filename("COM1"), "_COM1");
        assert_eq!(sanitize_filename("com1.txt"), "_com1.txt");
        assert_eq!(sanitize_filename("LPT9"), "_LPT9");
        assert_eq!(sanitize_filename("NUL"), "_NUL");
    }

    #[test]
    fn sanitize_does_not_prefix_com10() {
        assert_eq!(sanitize_filename("COM10.bin"), "COM10.bin");
    }

    #[test]
    fn sanitize_does_not_prefix_com0() {
        assert_eq!(sanitize_filename("COM0.exe"), "COM0.exe");
    }

    #[test]
    fn sanitize_prefixes_prn_stem() {
        assert_eq!(sanitize_filename("PRN.log"), "_PRN.log");
    }

    #[test]
    fn suggested_name_prefers_content_disposition() {
        assert_eq!(
            suggested_save_name_for_download(
                Some(r#"attachment; filename="Shipped.tgz""#),
                "https://cdn.example/dl?token=1"
            ),
            "Shipped.tgz"
        );
    }

    #[test]
    fn suggested_name_falls_back_to_last_path_segment() {
        assert_eq!(
            suggested_save_name_for_download(
                None,
                "https://files.example/a/b/archive.7z?x=1"
            ),
            "archive.7z"
        );
    }

    #[test]
    fn suggested_name_ipv6_url_path_segment() {
        assert_eq!(
            suggested_save_name_for_download(
                None,
                "https://[2001:db8::1]:443/pub/manual.pdf"
            ),
            "manual.pdf"
        );
    }

    #[test]
    fn suggested_name_unparseable_url() {
        assert_eq!(
            suggested_save_name_for_download(None, "not a valid URL"),
            "download"
        );
    }

    #[test]
    fn suggested_name_trailing_slash_uses_last_non_empty_segment() {
        assert_eq!(
            suggested_save_name_for_download(None, "https://example.com/path/"),
            "path"
        );
    }

    #[test]
    fn suggested_name_root_url_uses_download() {
        assert_eq!(
            suggested_save_name_for_download(None, "https://example.com/"),
            "download"
        );
    }

    #[test]
    fn suggested_name_whitespace_only_filename_falls_back() {
        assert_eq!(
            suggested_save_name_for_download(
                Some(r#"attachment; filename="   ""#),
                "https://x.example/y.iso"
            ),
            "y.iso"
        );
    }
}
