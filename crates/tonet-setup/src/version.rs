//! Semantic-ish version parsing and comparison for installer update checks.

use std::cmp::Ordering;

/// Parse `0.2.2`, `v0.2.2-preview`, etc. into comparable numeric components.
pub fn parse_version_parts(version: &str) -> Vec<u32> {
    let mut parts = Vec::new();
    for segment in version.trim().trim_start_matches('v').split(&['.', '-', '_'][..]) {
        if segment.is_empty() {
            continue;
        }
        let digits: String = segment.chars().take_while(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() {
            continue;
        }
        if let Ok(n) = digits.parse::<u32>() {
            parts.push(n);
        }
    }
    parts
}

pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let mut va = parse_version_parts(a);
    let mut vb = parse_version_parts(b);
    let len = va.len().max(vb.len());
    va.resize(len, 0);
    vb.resize(len, 0);
    va.cmp(&vb)
}

/// Installed copy is current or newer than the release we would fetch.
pub fn is_up_to_date(installed: &str, latest: &str) -> bool {
    compare_versions(installed, latest) != Ordering::Less
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orders_semver() {
        assert_eq!(compare_versions("0.2.1", "0.2.2"), Ordering::Less);
        assert_eq!(compare_versions("0.2.2", "0.2.2"), Ordering::Equal);
        assert_eq!(compare_versions("0.3.0", "0.2.9"), Ordering::Greater);
    }
}
