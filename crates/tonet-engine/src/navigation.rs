//! URL scheme checks for **fetched** page resources (HTTP stack), not internal `tonet://` routes.

/// Returns `true` if `url` may be loaded by the HTTP client (http/https only).
pub fn is_http_or_https_url(url: &str) -> bool {
    let t = url.trim();
    let lower = t.to_ascii_lowercase();
    lower.starts_with("http://") || lower.starts_with("https://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_https() {
        assert!(is_http_or_https_url("https://example.com"));
        assert!(is_http_or_https_url("HTTP://e.com"));
    }

    #[test]
    fn rejects_other_schemes() {
        assert!(!is_http_or_https_url("tonet://settings"));
        assert!(!is_http_or_https_url("ftp://x"));
    }
}
