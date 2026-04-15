//! Subset HTML read model for Tonet’s read-only view.
//!
//! [`parse_html`] runs the in-house tokenizer + tree builder, then flattens to [`DomNode`]s in
//! document order (`<title>`, `<h1>`, `<h2>`, `<p>`, `<a href>` with `http`/`https` after resolution).
//! Relative `href`s use the document URL, then the first `<base href>` in tree order when present.
//! A legacy linear scanner is kept as [`parse_html_scan`] for regression checks.
//!
//! HTML comments are ignored in the tree path; `<script>` / `<style>` use rawtext tokenization so
//! markup-like tokens inside do not become elements.

use url::Url;

use crate::html::tokenizer;
use crate::html::tree_builder::{self, ElementNode, Node};

/// Semantic kind of a node extracted from the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomNodeType {
    Title,
    H1,
    H2,
    Paragraph,
    Link,
}

impl DomNodeType {
    /// Lowercase HTML tag name for this kind.
    fn tag_name(self) -> &'static str {
        match self {
            DomNodeType::Title => "title",
            DomNodeType::H1 => "h1",
            DomNodeType::H2 => "h2",
            DomNodeType::Paragraph => "p",
            DomNodeType::Link => "a",
        }
    }

    /// All kinds Tonet recognizes for body content, in preferred detection order.
    fn all() -> [DomNodeType; 5] {
        [
            DomNodeType::Title,
            DomNodeType::H1,
            DomNodeType::H2,
            DomNodeType::Paragraph,
            DomNodeType::Link,
        ]
    }
}

/// Simplified DOM node for Tonet.
#[derive(Debug, Clone)]
pub struct DomNode {
    pub kind: DomNodeType,
    pub text: String,
    /// Absolute `http`/`https` URL when [`DomNodeType::Link`]; otherwise `None`.
    pub href: Option<String>,
}

/// Resolve `href` against `page_url` for navigation. Returns `None` for unsupported schemes or fragments-only.
pub fn resolve_href(page_url: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() || href.starts_with('#') {
        return None;
    }
    let lower = href.to_ascii_lowercase();
    if lower.starts_with("javascript:") || lower.starts_with("mailto:") || lower.starts_with("tel:") {
        return None;
    }

    if href.starts_with("//") {
        return Url::parse(&format!("https:{href}"))
            .ok()
            .filter(|u| matches!(u.scheme(), "http" | "https"))
            .map(|u| u.to_string());
    }

    if lower.starts_with("http://") || lower.starts_with("https://") {
        return Url::parse(href)
            .ok()
            .filter(|u| matches!(u.scheme(), "http" | "https"))
            .map(|u| u.to_string());
    }

    let base = Url::parse(page_url).ok()?;
    base.join(href)
        .ok()
        .filter(|u| matches!(u.scheme(), "http" | "https"))
        .map(|u| u.to_string())
}

/// Parse HTML via tokenizer → tree → [`DomNode`] stream. `page_url` resolves relative links.
pub fn parse_html(html: &str, page_url: &str) -> Vec<DomNode> {
    parse_html_from_tree(html, page_url)
}

/// Legacy linear scanner over raw HTML (kept for tests and diffing).
pub fn parse_html_scan(html: &str, page_url: &str) -> Vec<DomNode> {
    let mut out = Vec::new();
    let mut pos = 0usize;

    while pos < html.len() {
        let Some((idx, kind)) = find_next_target_open_tag(html, pos) else {
            break;
        };

        let tag = kind.tag_name();

        if kind == DomNodeType::Link {
            if let Some((href_abs, label, end_after_close)) = extract_anchor_link(html, idx, page_url) {
                out.push(DomNode {
                    kind: DomNodeType::Link,
                    text: label,
                    href: Some(href_abs),
                });
                pos = end_after_close;
            } else {
                pos = idx.saturating_add(1);
            }
            continue;
        }

        let Some((raw_inner, end_after_close)) = extract_inner_until_close(html, idx, tag) else {
            pos = idx.saturating_add(1);
            continue;
        };

        let cleaned = normalize_whitespace(&strip_html_tags(raw_inner));
        if !cleaned.is_empty() {
            out.push(DomNode {
                kind,
                text: cleaned,
                href: None,
            });
        }

        pos = end_after_close;
    }

    out
}

/// Tokenizer + tree builder → same [`DomNode`] shape as [`parse_html`].
pub fn parse_html_from_tree(html: &str, page_url: &str) -> Vec<DomNode> {
    let tokens = tokenizer::tokenize(html);
    let fragment = tree_builder::build_fragment(&tokens);
    let link_base = first_base_href_resolved(&fragment.children, page_url).unwrap_or_else(|| page_url.to_string());
    let mut out = Vec::new();
    for n in &fragment.children {
        walk_tree_to_dom(n, &link_base, &mut out);
    }
    out
}

/// First `<base href>` in depth-first document order that resolves to `http`/`https`, else `None`.
fn first_base_href_resolved(nodes: &[Node], page_url: &str) -> Option<String> {
    for n in nodes {
        let Node::Element(el) = n else {
            continue;
        };
        if el.name == "base" {
            if let Some(raw) = el.attrs.iter().find(|a| a.name == "href").map(|a| a.value.as_str()) {
                if let Some(abs) = resolve_href(page_url, raw) {
                    return Some(abs);
                }
            }
        }
        if let Some(u) = first_base_href_resolved(&el.children, page_url) {
            return Some(u);
        }
    }
    None
}

fn walk_tree_to_dom(node: &Node, link_base: &str, out: &mut Vec<DomNode>) {
    let Node::Element(el) = node else {
        return;
    };
    if el.name == "script" || el.name == "style" {
        return;
    }
    match el.name.as_str() {
        "title" => emit_dom_block(DomNodeType::Title, el, out),
        "h1" => emit_dom_block(DomNodeType::H1, el, out),
        "h2" => emit_dom_block(DomNodeType::H2, el, out),
        "p" => emit_dom_block(DomNodeType::Paragraph, el, out),
        "a" => emit_dom_link(el, link_base, out),
        _ => {
            for c in &el.children {
                walk_tree_to_dom(c, link_base, out);
            }
        }
    }
}

fn emit_dom_block(kind: DomNodeType, el: &ElementNode, out: &mut Vec<DomNode>) {
    let cleaned = normalize_whitespace(&collect_element_text(el));
    if !cleaned.is_empty() {
        out.push(DomNode {
            kind,
            text: cleaned,
            href: None,
        });
    }
}

fn emit_dom_link(el: &ElementNode, link_base: &str, out: &mut Vec<DomNode>) {
    let href_raw = el
        .attrs
        .iter()
        .find(|a| a.name == "href")
        .map(|a| a.value.as_str())
        .unwrap_or("");
    let Some(href_abs) = resolve_href(link_base, href_raw) else {
        return;
    };
    let mut label = normalize_whitespace(&collect_element_text(el));
    if label.is_empty() {
        label = href_raw.to_string();
    }
    out.push(DomNode {
        kind: DomNodeType::Link,
        text: label,
        href: Some(href_abs),
    });
}

fn collect_element_text(el: &ElementNode) -> String {
    let mut s = String::new();
    for c in &el.children {
        match c {
            Node::Text(t) => s.push_str(t),
            Node::Element(child) => {
                if child.name == "script" || child.name == "style" {
                    continue;
                }
                s.push_str(&collect_element_text(child));
            }
        }
    }
    s
}

/// Strips HTML tags from a fragment, keeping approximate visible text.
fn strip_html_tags(fragment: &str) -> String {
    let mut out = String::with_capacity(fragment.len());
    let mut in_tag = false;
    for ch in fragment.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

/// Collapses consecutive whitespace and trims ends.
fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn extract_anchor_link(html: &str, open_idx: usize, page_url: &str) -> Option<(String, String, usize)> {
    let bytes = html.as_bytes();
    let gt = find_byte(bytes, b'>', open_idx)?;
    let open_tag = html.get(open_idx..=gt)?;
    let href_raw = parse_href_from_opening_a(open_tag)?;
    let inner_start = gt + 1;
    let close_start = find_closing_tag(html, inner_start, "a")?;
    let tail = html.get(close_start..)?;
    let rel_gt = tail.find('>')?;
    let close_end = close_start + rel_gt + 1;
    let inner = html.get(inner_start..close_start)?;
    let mut label = normalize_whitespace(&strip_html_tags(inner));
    if label.is_empty() {
        label = href_raw.clone();
    }
    let href_abs = resolve_href(page_url, &href_raw)?;
    Some((href_abs, label, close_end))
}

fn parse_href_from_opening_a(open_tag: &str) -> Option<String> {
    let lower: String = open_tag.chars().map(|c| c.to_ascii_lowercase()).collect();
    let key = "href";
    let i = lower.find(key)? + key.len();
    let mut rest = open_tag.get(i..)?.trim_start();
    rest = rest.strip_prefix('=')?.trim_start();
    let val = if let Some(r) = rest.strip_prefix('"') {
        r.split('"').next()?.trim().to_string()
    } else if let Some(r) = rest.strip_prefix('\'') {
        r.split('\'').next()?.trim().to_string()
    } else {
        let end = rest
            .char_indices()
            .find(|(_, c)| c.is_whitespace() || *c == '>')
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        rest.get(..end)?.trim().to_string()
    };
    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

/// If `i` is at `<!--`, returns the index after `-->`; otherwise `None`.
fn skip_after_html_comment(html: &str, i: usize) -> Option<usize> {
    let b = html.as_bytes();
    if i + 3 >= b.len() {
        return None;
    }
    if b[i] != b'<' || b.get(i + 1) != Some(&b'!') || b.get(i + 2) != Some(&b'-') || b.get(i + 3) != Some(&b'-') {
        return None;
    }
    let mut j = i + 4;
    while j + 2 < b.len() {
        if b[j] == b'-' && b[j + 1] == b'-' && b[j + 2] == b'>' {
            return Some(j + 3);
        }
        j += 1;
    }
    Some(html.len())
}

/// Skip `<script`…`</script>` and `<style`…`</style>` so inner `<p>`-like text is not scanned.
fn skip_after_rawtext_element(html: &str, i: usize) -> Option<usize> {
    for tag in ["script", "style"] {
        if open_tag_matches(html, i, tag) {
            let gt = find_byte(html.as_bytes(), b'>', i)?;
            let search_from = gt + 1;
            let closing = format!("</{tag}>");
            let cl = closing.len();
            let mut j = search_from;
            while j + cl <= html.len() {
                if html[j..j + cl].eq_ignore_ascii_case(&closing) {
                    return Some(j + cl);
                }
                j += 1;
            }
            return Some(html.len());
        }
    }
    None
}

/// Finds the next supported open tag from `pos` (ASCII case-insensitive).
fn find_next_target_open_tag(html: &str, pos: usize) -> Option<(usize, DomNodeType)> {
    let bytes = html.as_bytes();
    let mut i = pos;
    while i < bytes.len() {
        if bytes[i] == b'<' {
            if let Some(after) = skip_after_html_comment(html, i) {
                i = after;
                continue;
            }
            if let Some(after) = skip_after_rawtext_element(html, i) {
                i = after;
                continue;
            }
            for kind in DomNodeType::all() {
                if open_tag_matches(html, i, kind.tag_name()) {
                    return Some((i, kind));
                }
            }
        }
        i += 1;
    }
    None
}

/// True if `idx` starts `<tag` followed by end of tag name (`>`, space, `/`).
fn open_tag_matches(html: &str, idx: usize, tag: &str) -> bool {
    let Some(rest) = html.get(idx..) else {
        return false;
    };
    let mut pattern = String::with_capacity(tag.len() + 1);
    pattern.push('<');
    pattern.push_str(tag);
    let prefix = match rest.get(..pattern.len().min(rest.len())) {
        Some(p) => p,
        None => return false,
    };
    if !eq_ignore_ascii_case_prefix(prefix, &pattern) {
        return false;
    }
    let after = idx + pattern.len();
    let c = html.as_bytes().get(after).copied();
    matches!(c, Some(b'>') | Some(b'/') | Some(b' ') | Some(b'\t') | Some(b'\n') | Some(b'\r'))
}

fn eq_ignore_ascii_case_prefix(a: &str, b: &str) -> bool {
    if a.len() < b.len() {
        return false;
    }
    a[..b.len()].eq_ignore_ascii_case(b)
}

/// Extracts inner HTML between the open tag at `open_idx` and `</tag>`.
/// Returns (raw inner, index after the closing `>`).
fn extract_inner_until_close<'a>(html: &'a str, open_idx: usize, tag: &str) -> Option<(&'a str, usize)> {
    let bytes = html.as_bytes();
    let gt = find_byte(bytes, b'>', open_idx)?;
    let inner_start = gt + 1;
    let close_start = find_closing_tag(html, inner_start, tag)?;
    let tail = html.get(close_start..)?;
    let rel_gt = tail.find('>')?;
    let close_end = close_start + rel_gt + 1;
    Some((html.get(inner_start..close_start)?, close_end))
}

fn find_byte(bytes: &[u8], needle: u8, from: usize) -> Option<usize> {
    bytes[from..].iter().position(|&b| b == needle).map(|p| from + p)
}

/// Finds `</tag>` (ASCII case-insensitive) starting at `from`.
fn find_closing_tag(html: &str, from: usize, tag: &str) -> Option<usize> {
    let needle = format!("</{tag}>");
    let bytes = html.as_bytes();
    let mut i = from;
    while i + 2 < bytes.len() {
        if bytes[i] == b'<' && bytes.get(i + 1) == Some(&b'/') {
            let rest = html.get(i..)?;
            if rest.len() >= needle.len() {
                let slice = &rest[..needle.len()];
                if slice.eq_ignore_ascii_case(needle.as_str()) {
                    return Some(i);
                }
            }
        }
        i += 1;
    }
    None
}

/// Extract all favicon candidate URLs from `<link>` tags in the HTML `<head>`,
/// followed by classic fallback paths. Returned in priority order.
pub fn extract_favicon_candidates(html: &str, page_url: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    let mut pos = 0usize;
    let bytes = html.as_bytes();

    while pos + 5 < bytes.len() {
        if bytes[pos] != b'<' {
            pos += 1;
            continue;
        }

        if matches_tag_prefix(bytes, pos, b"link") {
            if let Some(href) = extract_link_favicon_href(html, pos) {
                if let Some(abs) = resolve_favicon_href(page_url, &href) {
                    if !candidates.contains(&abs) {
                        candidates.push(abs);
                    }
                }
            }
        }

        if matches_tag_prefix(bytes, pos, b"/head")
            || matches_tag_prefix(bytes, pos, b"body")
        {
            break;
        }

        pos += 1;
    }

    if let Ok(base) = Url::parse(page_url) {
        let origin = base.origin().unicode_serialization();
        for path in ["/favicon.ico", "/favicon.svg", "/apple-touch-icon.png"] {
            let fallback = format!("{origin}{path}");
            if !candidates.contains(&fallback) {
                candidates.push(fallback);
            }
        }
    }

    candidates
}

fn matches_tag_prefix(bytes: &[u8], pos: usize, tag: &[u8]) -> bool {
    let start = pos + 1;
    if start + tag.len() > bytes.len() {
        return false;
    }
    for (j, &expected) in tag.iter().enumerate() {
        if bytes[start + j].to_ascii_lowercase() != expected {
            return false;
        }
    }
    let after = start + tag.len();
    if after >= bytes.len() {
        return false;
    }
    matches!(bytes[after], b'>' | b'/' | b' ' | b'\t' | b'\n' | b'\r')
}

fn extract_link_favicon_href(html: &str, open_idx: usize) -> Option<String> {
    let gt = find_byte(html.as_bytes(), b'>', open_idx)?;
    let open_tag = html.get(open_idx..=gt)?;
    let lower = open_tag.to_ascii_lowercase();

    if !lower.contains("rel") {
        return None;
    }

    let is_icon = lower.contains("icon")
        || lower.contains("shortcut")
        || lower.contains("apple-touch-icon");
    if !is_icon {
        return None;
    }

    parse_attr_value(open_tag, "href")
}

fn parse_attr_value(tag: &str, attr: &str) -> Option<String> {
    let lower: String = tag.chars().map(|c| c.to_ascii_lowercase()).collect();
    let needle = attr;
    let found = lower.find(needle)?;
    let after_key = found + needle.len();
    let rest = tag.get(after_key..)?.trim_start();
    let rest = rest.strip_prefix('=')?;
    let rest = rest.trim_start();

    let val = if let Some(r) = rest.strip_prefix('"') {
        r.split('"').next()?.trim().to_string()
    } else if let Some(r) = rest.strip_prefix('\'') {
        r.split('\'').next()?.trim().to_string()
    } else {
        let end = rest
            .char_indices()
            .find(|(_, c)| c.is_whitespace() || *c == '>')
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        rest.get(..end)?.trim().to_string()
    };

    if val.is_empty() {
        None
    } else {
        Some(val)
    }
}

fn resolve_favicon_href(page_url: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() {
        return None;
    }
    if href.starts_with("data:") {
        return Some(href.to_string());
    }
    resolve_href(page_url, href)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_title_and_paragraph() {
        let html = "<html><title>Hello</title><body><p>World text</p></body>";
        let nodes = parse_html(html, "https://example.com/");
        assert!(nodes.iter().any(|n| n.kind == DomNodeType::Title && n.text == "Hello"));
        assert!(nodes.iter().any(|n| n.kind == DomNodeType::Paragraph && n.text == "World text"));
    }

    #[test]
    fn resolves_relative_link() {
        let html = r#"<a href="/path">Go</a>"#;
        let nodes = parse_html(html, "https://example.com/page");
        let link = nodes.iter().find(|n| n.kind == DomNodeType::Link).expect("link");
        assert_eq!(link.href.as_deref(), Some("https://example.com/path"));
        assert_eq!(link.text, "Go");
    }

    #[test]
    fn first_base_href_wins_for_relative_links() {
        let html = r#"<html><head>
            <base href="https://cdn.example.com/assets/">
            <base href="https://ignore.example/">
            </head><body><a href="page.html">x</a></body></html>"#;
        let nodes = parse_html(html, "https://origin.example.com/dir/doc.html");
        let link = nodes.iter().find(|n| n.kind == DomNodeType::Link).expect("link");
        assert_eq!(
            link.href.as_deref(),
            Some("https://cdn.example.com/assets/page.html")
        );
    }

    #[test]
    fn absolute_https_link_unchanged() {
        let html = r#"<a href="https://other.test/x">Y</a>"#;
        let nodes = parse_html(html, "https://example.com/");
        let link = nodes.iter().find(|n| n.kind == DomNodeType::Link).unwrap();
        assert_eq!(link.href.as_deref(), Some("https://other.test/x"));
    }

    #[test]
    fn skips_javascript_href() {
        let html = r#"<a href="javascript:void(0)">X</a>"#;
        let nodes = parse_html(html, "https://example.com/");
        assert!(!nodes.iter().any(|n| n.kind == DomNodeType::Link));
    }

    #[test]
    fn resolve_href_protocol_relative() {
        let u = resolve_href("https://ex.com/a", "//cdn.ex.com/z").unwrap();
        assert_eq!(u, "https://cdn.ex.com/z");
    }

    #[test]
    fn extracts_favicon_from_link_tag() {
        let html = r#"<html><head>
            <link rel="icon" type="image/png" href="/static/favicon-32.png">
            <link rel="shortcut icon" href="/old-fav.ico">
        </head><body></body></html>"#;
        let cands = extract_favicon_candidates(html, "https://example.com/page");
        assert!(cands[0].contains("favicon-32.png"));
        assert!(cands[1].contains("old-fav.ico"));
        assert!(cands.iter().any(|c| c.ends_with("/favicon.ico")));
    }

    #[test]
    fn comment_does_not_emit_nested_paragraph() {
        let html = "<body><!-- <p>ghost</p> --><p>real</p>";
        let nodes = parse_html(html, "https://example.com/");
        assert_eq!(
            nodes.iter().filter(|n| n.kind == DomNodeType::Paragraph).count(),
            1
        );
        assert!(nodes.iter().any(|n| n.kind == DomNodeType::Paragraph && n.text == "real"));
    }

    #[test]
    fn script_body_does_not_emit_paragraph() {
        let html = r#"<body><script>var s = "<p>inside</p>";</script><p>after</p>"#;
        let nodes = parse_html(html, "https://example.com/");
        assert_eq!(
            nodes.iter().filter(|n| n.kind == DomNodeType::Paragraph).count(),
            1
        );
        assert!(nodes.iter().any(|n| n.kind == DomNodeType::Paragraph && n.text == "after"));
    }
}
