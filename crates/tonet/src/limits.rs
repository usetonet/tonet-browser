/// Hard budgets for networking and document ingestion. Align with [`crate::policy`].
///
/// Values match the current Tonet MVP defaults; tune via a single place as the engine grows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EngineLimits {
    /// Maximum UTF-8/HTML document size accepted after a successful HTTP 200 (bytes).
    pub max_document_bytes: usize,
    /// Maximum favicon payload size (bytes).
    pub max_favicon_bytes: usize,
    /// Timeout for primary page `GET` (seconds).
    pub http_request_timeout_secs: u64,
    /// Timeout for favicon probe requests (seconds).
    pub favicon_request_timeout_secs: u64,
    /// Maximum HTTP redirects to follow for a navigation (`0` means do not follow redirects).
    pub max_http_redirects: u32,
    /// Cap on `<link rel=stylesheet>` fetches after a navigation (each obeys [`Self::max_document_bytes`]).
    pub max_stylesheets_per_document: usize,
}

impl EngineLimits {
    /// Production-aligned defaults used by the desktop shell today.
    pub const STANDARD: Self = Self {
        max_document_bytes: 1_000_000,
        max_favicon_bytes: 512_000,
        http_request_timeout_secs: 45,
        favicon_request_timeout_secs: 8,
        // Explicit cap (reqwest previously followed up to 10 by default; now centralized here).
        max_http_redirects: 10,
        max_stylesheets_per_document: 16,
    };
}

impl Default for EngineLimits {
    fn default() -> Self {
        Self::STANDARD
    }
}

#[cfg(test)]
mod tests {
    use super::EngineLimits;

    #[test]
    fn standard_matches_documented_mvp_defaults() {
        let s = EngineLimits::STANDARD;
        assert_eq!(s.max_document_bytes, 1_000_000);
        assert_eq!(s.max_favicon_bytes, 512_000);
        assert_eq!(s.http_request_timeout_secs, 45);
        assert_eq!(s.favicon_request_timeout_secs, 8);
        assert_eq!(s.max_http_redirects, 10);
        assert_eq!(s.max_stylesheets_per_document, 16);
    }

    #[test]
    fn default_matches_standard() {
        assert_eq!(EngineLimits::default(), EngineLimits::STANDARD);
    }
}
