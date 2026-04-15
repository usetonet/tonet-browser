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
    /// Maximum HTTP redirects to follow for a navigation (0 = none yet; reserved).
    pub max_http_redirects: u32,
}

impl EngineLimits {
    /// Production-aligned defaults used by the desktop shell today.
    pub const STANDARD: Self = Self {
        max_document_bytes: 1_000_000,
        max_favicon_bytes: 512_000,
        http_request_timeout_secs: 45,
        favicon_request_timeout_secs: 8,
        max_http_redirects: 0,
    };
}

impl Default for EngineLimits {
    fn default() -> Self {
        Self::STANDARD
    }
}
