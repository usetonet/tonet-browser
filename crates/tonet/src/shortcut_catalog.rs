//! Chromium-style shortcut reference for the internal settings page (display only).
//! Data is loaded from `assets/shortcuts.tsv` (tab-separated: command, binding).

use std::sync::LazyLock;

static CATALOG: LazyLock<Vec<(String, String)>> = LazyLock::new(|| {
    let raw = include_str!("../assets/shortcuts.tsv");
    let mut out = Vec::new();
    for line in raw.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(2, '\t');
        let cmd = parts.next().unwrap_or("").trim();
        if cmd.is_empty() {
            continue;
        }
        let bind = parts
            .next()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or("—")
            .to_string();
        out.push((cmd.to_string(), bind));
    }
    out
});

pub fn filter_pairs(query: &str) -> Vec<(String, String)> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return CATALOG.clone();
    }
    CATALOG
        .iter()
        .filter(|(cmd, keys)| {
            cmd.to_ascii_lowercase().contains(&q) || keys.to_ascii_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}
