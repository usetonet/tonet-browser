//! Heuristics for [`WebViewDelegate::load_web_resource`] (Servo): when to intercept a navigation
//! and hand it to Tonet’s own fetch + save-as path (no cookies / auth from Servo’s jar — see checklist).
//! Only **`http` / `https`** URLs are eligible; `ws` / `wss` / `file` / `ftp` / … are ignored.
//! Only **`GET`** is intercepted (not `POST` / `HEAD` / `OPTIONS` / `PATCH` / …).
//!
//! **HEAD probe (narrow):** when the URL’s last path segment has **no** dotted extension (e.g. no `.zip`)
//! but is named like a download endpoint (`download` / `export` / `attachment`), Tonet may issue a short
//! **`HEAD`** (see `background_download::head_suggests_intercept_binary_get`) to read
//! `Content-Disposition` / `Content-Type` before intercepting — avoids probing normal extensionless pages.

use http::Method;
use url::Url;

/// Extensions for **main-frame** navigations we treat as “save file” rather than in-page render.
/// Conservative: no `html`/`htm`, no common in-browser media (`png`, `mp4`, …).
const MAIN_FRAME_DOWNLOAD_EXTENSIONS: &[&str] = &[
    "7z", "apk", "bin", "bz2", "csv", "dat", "deb", "dmg", "doc", "docx", "exe", "gz", "iso",
    "msi", "odb", "odp", "ods", "odt", "pdf", "ppt", "pptx", "rar", "rpm", "tar", "tgz",
    "torrent", "wasm", "xls", "xlsx", "xz", "zip",
];

#[inline]
fn path_extension_lower(url: &Url) -> Option<String> {
    let seg = super::url_path::last_non_empty_path_segment(url)?;
    let (_, ext) = seg.rsplit_once('.')?;
    if ext.is_empty() {
        return None;
    }
    Some(ext.to_ascii_lowercase())
}

#[inline]
fn extension_triggers_download(ext: &str) -> bool {
    MAIN_FRAME_DOWNLOAD_EXTENSIONS.iter().any(|&e| e == ext)
}

/// Same rule as extension allowlist (for reuse after parsing `Content-Disposition` filenames).
#[inline]
pub(crate) fn is_allowlisted_download_extension(ext: &str) -> bool {
    extension_triggers_download(ext.trim())
}

/// Last URL path segment names that may hide a real filename in **`Content-Disposition`** (extensionless URL).
const HEAD_PROBE_LAST_SEGMENTS: &[&str] = &["download", "export", "attachment"];

/// `true` when a **short `HEAD`** may disambiguate extensionless “download-shaped” URLs (see module docs).
#[inline]
pub(crate) fn should_head_probe_main_frame_binary_get(
    method: &Method,
    url: &Url,
    is_for_main_frame: bool,
    is_redirect: bool,
) -> bool {
    if is_redirect || !is_for_main_frame || method != &Method::GET {
        return false;
    }
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    if path_extension_lower(url).is_some() {
        return false;
    }
    let Some(seg) = super::url_path::last_non_empty_path_segment(url) else {
        return false;
    };
    HEAD_PROBE_LAST_SEGMENTS
        .iter()
        .any(|s| seg.eq_ignore_ascii_case(s))
}

/// `true` when Tonet should [`servo::WebResourceLoad::intercept`] and run a background save.
/// **GET** only (see module docs).
#[inline]
pub(crate) fn should_intercept_main_frame_binary_get(
    method: &Method,
    url: &Url,
    is_for_main_frame: bool,
    is_redirect: bool,
) -> bool {
    if is_redirect || !is_for_main_frame {
        return false;
    }
    if method != &Method::GET {
        return false;
    }
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    path_extension_lower(url)
        .as_deref()
        .map(extension_triggers_download)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn u(s: &str) -> Url {
        Url::parse(s).unwrap()
    }

    #[test]
    fn zip_main_get_yes() {
        assert!(should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/files/app.zip"),
            true,
            false,
        ));
    }

    #[test]
    fn html_main_get_no() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/page.html"),
            true,
            false,
        ));
    }

    #[test]
    fn zip_subresource_no() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/asset.zip"),
            false,
            false,
        ));
    }

    #[test]
    fn redirect_no() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/a.zip"),
            true,
            true,
        ));
    }

    #[test]
    fn post_no() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::POST,
            &u("https://example.com/a.zip"),
            true,
            false,
        ));
    }

    #[test]
    fn head_no() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::HEAD,
            &u("https://example.com/a.zip"),
            true,
            false,
        ));
    }

    #[test]
    fn options_put_delete_patch_no() {
        for method in [Method::OPTIONS, Method::PUT, Method::DELETE, Method::PATCH] {
            assert!(
                !should_intercept_main_frame_binary_get(&method, &u("https://example.com/a.zip"), true, false),
                "{method:?}"
            );
        }
    }

    #[test]
    fn pdf_main_get_yes() {
        assert!(should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://cdn.example.org/static/guide.PDF"),
            true,
            false,
        ));
    }

    #[test]
    fn zip_with_query_yes() {
        assert!(should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/dl/app.zip?token=abc"),
            true,
            false,
        ));
    }

    #[test]
    fn ftp_scheme_no_even_if_zip() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("ftp://files.example/pub/archive.zip"),
            true,
            false,
        ));
    }

    #[test]
    fn file_scheme_no() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("file:///C:/tmp/x.zip"),
            true,
            false,
        ));
    }

    #[test]
    fn ws_wss_schemes_no_even_with_zip_path() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("ws://echo.example/socket"),
            true,
            false,
        ));
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("wss://echo.example/downloads/file.zip"),
            true,
            false,
        ));
    }

    #[test]
    fn no_extension_last_segment_no_without_head() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/download"),
            true,
            false,
        ));
    }

    #[test]
    fn head_probe_yes_for_extensionless_download_segment() {
        assert!(should_head_probe_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/download"),
            true,
            false,
        ));
        assert!(should_head_probe_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/api/EXPORT"),
            true,
            false,
        ));
    }

    #[test]
    fn head_probe_no_when_url_has_extension() {
        assert!(!should_head_probe_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/download.html"),
            true,
            false,
        ));
    }

    #[test]
    fn head_probe_no_for_zip_url() {
        assert!(!should_head_probe_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/dl/app.zip"),
            true,
            false,
        ));
    }

    #[test]
    fn multi_dot_uses_last_extension_segment() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/build/release.final"),
            true,
            false,
        ));
        assert!(should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/build/release.final.zip"),
            true,
            false,
        ));
    }

    #[test]
    fn tar_gz_triggers_on_final_gz_suffix() {
        // `bundle.tar.gz` → suffix `gz` (allowlisted); we do not special-case `.tar.gz`.
        assert!(should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/out/bundle.tar.gz"),
            true,
            false,
        ));
    }

    #[test]
    fn trailing_slash_uses_prior_segment_for_extension() {
        assert!(should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/dl/app.zip/"),
            true,
            false,
        ));
    }

    #[test]
    fn root_path_no_intercept() {
        assert!(!should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://example.com/"),
            true,
            false,
        ));
    }

    #[test]
    fn ipv6_https_host_zip_yes() {
        assert!(should_intercept_main_frame_binary_get(
            &Method::GET,
            &u("https://[::1]:8443/files/app.zip"),
            true,
            false,
        ));
    }
}
