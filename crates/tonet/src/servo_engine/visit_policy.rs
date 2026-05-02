//! Pure helpers for Servo-driven **visit history** (no `WebView` / Win32).
#![cfg_attr(not(all(feature = "servo-engine", windows)), allow(dead_code))] // `runtime_win` is the only non-test consumer; it is not built on other targets.

/// URLs counted in the same bucket as Tonet’s HTML fetch history: `http` / `https` only.
#[inline]
pub(crate) fn is_http_or_https_history_url(url_trim: &str) -> bool {
    let t = url_trim.trim();
    t.starts_with("http://") || t.starts_with("https://")
}

/// Whether to append a [`crate::browser_log::BrowserLog::record_visit`] for this Servo snapshot.
#[inline]
pub(crate) fn should_record_visit(
    load_complete: bool,
    committed_url_trim: Option<&str>,
    was_complete_last_frame: bool,
    last_recorded_url: Option<&str>,
) -> bool {
    if !load_complete {
        return false;
    }
    let Some(u) = committed_url_trim.map(str::trim).filter(|s| !s.is_empty()) else {
        return false;
    };
    if !is_http_or_https_history_url(u) {
        return false;
    }
    let became_complete = !was_complete_last_frame;
    let url_changed = last_recorded_url != Some(u);
    became_complete || url_changed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_non_http() {
        assert!(!should_record_visit(true, Some("about:blank"), false, None,));
        assert!(!should_record_visit(
            true,
            Some(" tonet://settings "),
            false,
            None,
        ));
    }

    #[test]
    fn first_complete_records() {
        assert!(should_record_visit(
            true,
            Some("https://a.example/"),
            false,
            None,
        ));
    }

    #[test]
    fn stable_complete_same_url_skips() {
        assert!(!should_record_visit(
            true,
            Some("https://a.example/"),
            true,
            Some("https://a.example/"),
        ));
    }

    #[test]
    fn reload_same_url_records_after_incomplete() {
        assert!(should_record_visit(
            true,
            Some("https://a.example/"),
            false,
            Some("https://a.example/"),
        ));
    }

    #[test]
    fn spa_url_change_while_complete_records() {
        assert!(should_record_visit(
            true,
            Some("https://a.example/b"),
            true,
            Some("https://a.example/a"),
        ));
    }

    #[test]
    fn history_url_accepts_http_https_trimmed() {
        assert!(is_http_or_https_history_url("https://example/"));
        assert!(is_http_or_https_history_url("  http://x "));
        assert!(is_http_or_https_history_url("\thttps://y/z\n"));
    }

    #[test]
    fn history_url_accepts_ipv6_literal_host() {
        assert!(is_http_or_https_history_url("https://[::1]/page"));
        assert!(is_http_or_https_history_url("http://[2001:db8::1]:8080/"));
    }

    #[test]
    fn should_record_visit_accepts_ipv6_https_url() {
        assert!(should_record_visit(
            true,
            Some("https://[::1]/index.html"),
            false,
            None,
        ));
    }

    #[test]
    fn history_url_rejects_non_http_scheme() {
        assert!(!is_http_or_https_history_url("about:blank"));
        assert!(!is_http_or_https_history_url("ftp://a"));
        assert!(!is_http_or_https_history_url(""));
        assert!(!is_http_or_https_history_url("   "));
    }

    #[test]
    fn history_url_rejects_embedder_schemes() {
        assert!(!is_http_or_https_history_url("javascript:void(0)"));
        assert!(!is_http_or_https_history_url("data:text/plain,hi"));
        assert!(!is_http_or_https_history_url("blob:https://x.example/uuid"));
        assert!(!is_http_or_https_history_url("file:///tmp/x"));
    }

    #[test]
    fn history_url_rejects_ws_wss_and_chrome() {
        assert!(!is_http_or_https_history_url("ws://echo.example/"));
        assert!(!is_http_or_https_history_url("wss://echo.example/"));
        assert!(!is_http_or_https_history_url("chrome://settings/"));
    }

    #[test]
    fn should_record_visit_skips_javascript_and_data() {
        assert!(!should_record_visit(
            true,
            Some("javascript:history.back()"),
            false,
            None,
        ));
        assert!(!should_record_visit(
            true,
            Some("data:text/html,<p>x</p>"),
            false,
            None,
        ));
    }

    /// Scheme match is ASCII case-sensitive (`starts_with`), matching typical URL serialization.
    #[test]
    fn history_url_rejects_uppercase_http_scheme() {
        assert!(!is_http_or_https_history_url("HTTP://example.com"));
        assert!(!is_http_or_https_history_url("HTTPS://example.com"));
    }

    #[test]
    fn should_record_visit_trims_committed_url() {
        assert!(should_record_visit(
            true,
            Some("  https://trimmed.example/  "),
            false,
            None,
        ));
    }

    #[test]
    fn incomplete_load_never_records() {
        assert!(!should_record_visit(
            false,
            Some("https://a.example/"),
            false,
            None,
        ));
    }

    #[test]
    fn missing_or_empty_committed_url_never_records() {
        assert!(!should_record_visit(true, None, false, None));
        assert!(!should_record_visit(true, Some(""), false, None));
        assert!(!should_record_visit(true, Some("   \t"), false, None));
    }

    #[test]
    fn trimmed_empty_after_trim_skips() {
        assert!(!should_record_visit(true, Some("  "), true, None));
    }

    /// [`should_record_visit`] compares `last_recorded_url` to the **trimmed** committed URL
    /// with plain `Option` equality — callers should store the same normalized string to avoid a
    /// second history row for the same navigation.
    #[test]
    fn should_record_visit_last_recorded_whitespace_diff_counts_as_url_change() {
        assert!(should_record_visit(
            true,
            Some("https://a.example/"),
            true,
            Some(" https://a.example/ "),
        ));
    }

    #[test]
    fn history_url_rejects_mailto_and_magnet() {
        assert!(!is_http_or_https_history_url("mailto:a@b.example"));
        assert!(!is_http_or_https_history_url("magnet:?xt=urn:btih:abc"));
    }

    #[test]
    fn should_record_visit_skips_mailto_even_when_complete() {
        assert!(!should_record_visit(
            true,
            Some("mailto:user@example.org"),
            false,
            None,
        ));
    }
}
