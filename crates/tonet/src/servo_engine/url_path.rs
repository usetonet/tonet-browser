//! Shared URL path helpers for Servo download flows (`background_download`, `download_heuristic`).

use url::Url;

/// Ignores a trailing empty segment from a final `/` (e.g. `…/dir/` → `Some("dir")`).
#[inline]
pub(crate) fn last_non_empty_path_segment(url: &Url) -> Option<&str> {
    url.path_segments()?.rev().find(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn skips_trailing_slash_segment() {
        let u = Url::parse("https://example.com/releases/").unwrap();
        assert_eq!(last_non_empty_path_segment(&u), Some("releases"));
    }

    #[test]
    fn bare_origin_path() {
        let u = Url::parse("https://example.com/").unwrap();
        assert!(last_non_empty_path_segment(&u).is_none());
    }

    #[test]
    fn single_path_segment() {
        let u = Url::parse("https://cdn.example/assets/archive.tgz").unwrap();
        assert_eq!(last_non_empty_path_segment(&u), Some("archive.tgz"));
    }

    #[test]
    fn ipv6_host_last_segment() {
        let u = Url::parse("https://[::1]:8443/dl/setup.msi").unwrap();
        assert_eq!(last_non_empty_path_segment(&u), Some("setup.msi"));
    }

    #[test]
    fn skips_empty_segments_from_double_slash() {
        let u = Url::parse("https://example.com/a//b//file.zip").unwrap();
        assert_eq!(last_non_empty_path_segment(&u), Some("file.zip"));
    }
}
