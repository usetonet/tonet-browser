//! Minimal **author stylesheet** cascade: single **type** selectors only (`p`, `h1`, …).
//!
//! No combinators, classes, IDs, or pseudo-classes. Matched rules apply in source order; the last
//! declaration wins per property name (ASCII-lowercased), across all stylesheets in `bundle` order.

use std::collections::HashMap;

use super::declarations::ParsedQualifiedRule;

/// `true` when `prelude` is one ASCII type-like token equal to `element_tag_lower` (already lowercased).
pub fn prelude_matches_simple_type(prelude: &str, element_tag_lower: &str) -> bool {
    let mut parts = prelude.split_whitespace();
    let Some(first) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    let sel = first;
    if sel.is_empty() || !selector_token_is_simple_type(sel) {
        return false;
    }
    sel.eq_ignore_ascii_case(element_tag_lower)
}

fn selector_token_is_simple_type(sel: &str) -> bool {
    if sel.bytes().any(|b| {
        matches!(
            b,
            b',' | b'>' | b'+' | b'~' | b'#' | b'.' | b'[' | b':' | b'(' | b'*' | b'"' | b'\''
        )
    }) {
        return false;
    }
    let mut it = sel.bytes();
    let Some(b0) = it.next() else {
        return false;
    };
    if !b0.is_ascii_alphabetic() {
        return false;
    }
    it.all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// For one element tag, collect declarations from every matching simple-type rule; **last wins** per property.
pub fn cascade_simple_type_rules(
    bundle: &[(String, Vec<ParsedQualifiedRule>)],
    element_tag_lower: &str,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    for (_url, rules) in bundle {
        for rule in rules {
            if prelude_matches_simple_type(&rule.prelude_display, element_tag_lower) {
                for d in &rule.declarations {
                    let key = d.property.to_ascii_lowercase();
                    out.insert(key, d.value_display.clone());
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::declarations::ParsedQualifiedRule;
    use crate::css::declarations::SimpleDeclaration;

    fn rule(prelude: &str, decls: Vec<(&str, &str)>) -> ParsedQualifiedRule {
        ParsedQualifiedRule {
            prelude_display: prelude.into(),
            declarations: decls
                .into_iter()
                .map(|(p, v)| SimpleDeclaration {
                    property: p.into(),
                    value_display: v.into(),
                })
                .collect(),
        }
    }

    #[test]
    fn prelude_p_whitespace() {
        assert!(prelude_matches_simple_type("  p ", "p"));
        assert!(!prelude_matches_simple_type("p.intro", "p"));
        assert!(!prelude_matches_simple_type("div p", "p"));
    }

    #[test]
    fn cascade_last_wins_same_property() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule("p", vec![("color", "red")]),
                rule("p", vec![("color", "blue")]),
            ],
        )];
        let m = cascade_simple_type_rules(&bundle, "p");
        assert_eq!(m.get("color").map(String::as_str), Some("blue"));
    }

    #[test]
    fn cascade_across_sheets_in_order() {
        let bundle = vec![
            (
                "https://x/1.css".into(),
                vec![rule("h1", vec![("font-size", "10px")])],
            ),
            (
                "https://x/2.css".into(),
                vec![rule("h1", vec![("font-size", "20px")])],
            ),
        ];
        let m = cascade_simple_type_rules(&bundle, "h1");
        assert_eq!(m.get("font-size").map(String::as_str), Some("20px"));
    }
}
