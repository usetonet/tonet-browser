//! Top-level HTTP(S) document URL rules **before** any network I/O.
//!
//! Keeps policy in the Tonet crate so the shell and future loaders share one definition.

use std::fmt;

/// Hard cap on URL length for a main-document GET (defensive; unrelated to HTML size limits).
pub const MAX_DOCUMENT_URL_BYTES: usize = 8192;

/// URL rejected by [`normalize_document_url_for_http_get`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentUrlError {
    Empty,
    TooLong,
    SchemeNotHttp,
}

impl fmt::Display for DocumentUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DocumentUrlError::Empty => write!(f, "URL is empty"),
            DocumentUrlError::TooLong => write!(f, "URL exceeds {} bytes", MAX_DOCUMENT_URL_BYTES),
            DocumentUrlError::SchemeNotHttp => {
                write!(f, "Only http and https URLs are allowed for page loads")
            }
        }
    }
}

impl std::error::Error for DocumentUrlError {}

/// Trim and validate a URL string for a **main frame** HTTP GET.
///
/// Returns the trimmed string on success (caller may parse with `url::Url` or another parser).
#[inline]
pub fn normalize_document_url_for_http_get(raw: &str) -> Result<String, DocumentUrlError> {
    let t = raw.trim();
    if t.is_empty() {
        return Err(DocumentUrlError::Empty);
    }
    if t.len() > MAX_DOCUMENT_URL_BYTES {
        return Err(DocumentUrlError::TooLong);
    }
    let lower = t.to_ascii_lowercase();
    if !(lower.starts_with("http://") || lower.starts_with("https://")) {
        return Err(DocumentUrlError::SchemeNotHttp);
    }
    Ok(t.to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_document_url_for_http_get, DocumentUrlError, MAX_DOCUMENT_URL_BYTES,
    };

    #[test]
    fn accepts_http_https_trimmed() {
        assert_eq!(
            normalize_document_url_for_http_get("  https://example.test/x  ").unwrap(),
            "https://example.test/x"
        );
        assert_eq!(
            normalize_document_url_for_http_get("http://127.0.0.1/").unwrap(),
            "http://127.0.0.1/"
        );
    }

    #[test]
    fn rejects_empty_and_whitespace_only() {
        assert_eq!(
            normalize_document_url_for_http_get(""),
            Err(DocumentUrlError::Empty)
        );
        assert_eq!(
            normalize_document_url_for_http_get("  \t  "),
            Err(DocumentUrlError::Empty)
        );
    }

    #[test]
    fn rejects_non_http_scheme() {
        assert_eq!(
            normalize_document_url_for_http_get("ftp://files.example/x"),
            Err(DocumentUrlError::SchemeNotHttp)
        );
        assert_eq!(
            normalize_document_url_for_http_get("tonet://settings"),
            Err(DocumentUrlError::SchemeNotHttp)
        );
    }

    #[test]
    fn rejects_overlong_url() {
        let prefix = "https://";
        let mut raw = String::with_capacity(MAX_DOCUMENT_URL_BYTES + 8);
        raw.push_str(prefix);
        raw.push_str(&"a".repeat(MAX_DOCUMENT_URL_BYTES - prefix.len() + 1));
        assert!(raw.len() > MAX_DOCUMENT_URL_BYTES);
        assert_eq!(
            normalize_document_url_for_http_get(&raw),
            Err(DocumentUrlError::TooLong)
        );
    }

    #[test]
    fn upper_case_scheme_still_http_https_after_lower_check() {
        // ASCII lower ensures HTTP:// and HTTPS:// are accepted (normalized output keeps caller trim).
        assert_eq!(
            normalize_document_url_for_http_get("HTTPS://UP.EXAMPLE/").unwrap(),
            "HTTPS://UP.EXAMPLE/"
        );
    }
}
