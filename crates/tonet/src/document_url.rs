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
