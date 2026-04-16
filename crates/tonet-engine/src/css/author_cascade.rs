//! Minimal **author stylesheet** cascade: one **simple selector** per prelude (`p`, `.lead`, `#x`, `#x.y`, `p.lead`, `p#main`).
//!
//! No combinators or pseudo-classes. When several rules set the same property, the winner is the
//! higher **specificity** (`#id.class` > `tag#id` > `#id` > `tag.class` > `.class` > `type`); ties break by **document order** (later rule).
//!
//! [`cascade_document_defaults`] collects only `html` / `body` **type** rules in stylesheet order so
//! the shell can approximate inherited paint (Tonet does not emit `html` / `body` as `DomNode`s).

use std::collections::HashMap;

use super::declarations::ParsedQualifiedRule;

/// Parsed prelude: type, class, id, `#id.class`, compound `tag.class`, or compound `tag#id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SimpleSelectorPrelude {
    Type { tag: String },
    Class { name: String },
    Id { name: String },
    /// `#main.lead` — `element_id` **and** a matching `class` token (HTML case-sensitive).
    IdClass { id: String, class: String },
    /// `p.intro` — element must match `tag` **and** include `class` in its `class` list (HTML case-sensitive).
    TypeClass { tag: String, class: String },
    /// `p#main` — element must match `tag` **and** `element_id` (HTML case-sensitive).
    TypeId { tag: String, id: String },
}

fn specificity(p: &SimpleSelectorPrelude) -> (u8, u8, u32) {
    match p {
        SimpleSelectorPrelude::Type { .. } => (0, 0, 1),
        SimpleSelectorPrelude::Class { .. } => (0, 1, 0),
        SimpleSelectorPrelude::TypeClass { .. } => (0, 1, 1),
        SimpleSelectorPrelude::Id { .. } => (1, 0, 0),
        SimpleSelectorPrelude::TypeId { .. } => (1, 0, 1),
        SimpleSelectorPrelude::IdClass { .. } => (1, 1, 0),
    }
}

/// Parse a single-selector prelude after trimming internal runs of whitespace to one token.
pub fn parse_simple_prelude(prelude: &str) -> Option<SimpleSelectorPrelude> {
    let mut parts = prelude.split_whitespace();
    let first = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let s = first;
    if let Some(rest) = s.strip_prefix('#') {
        if rest.is_empty() {
            return None;
        }
        if let Some(dot_idx) = rest.find('.') {
            if dot_idx == 0 || dot_idx + 1 >= rest.len() {
                return None;
            }
            let id_part = &rest[..dot_idx];
            let cls = &rest[dot_idx + 1..];
            if !is_simple_ident(id_part) || !is_simple_ident(cls) {
                return None;
            }
            return Some(SimpleSelectorPrelude::IdClass {
                id: id_part.to_string(),
                class: cls.to_string(),
            });
        }
        if !is_simple_ident(rest) {
            return None;
        }
        return Some(SimpleSelectorPrelude::Id {
            name: rest.to_string(),
        });
    }
    if let Some(rest) = s.strip_prefix('.') {
        if rest.is_empty() || !is_simple_ident(rest) {
            return None;
        }
        return Some(SimpleSelectorPrelude::Class {
            name: rest.to_string(),
        });
    }
    if let Some(hash_idx) = s.find('#') {
        if hash_idx > 0 {
            let tag = &s[..hash_idx];
            let id = &s[hash_idx + 1..];
            if id.is_empty() || !is_simple_ident(id) || !selector_token_is_simple_type(tag) {
                return None;
            }
            return Some(SimpleSelectorPrelude::TypeId {
                tag: tag.to_ascii_lowercase(),
                id: id.to_string(),
            });
        }
    }
    if let Some(dot_idx) = s.find('.') {
        if dot_idx > 0 {
            let tag = &s[..dot_idx];
            let cls = &s[dot_idx + 1..];
            if cls.is_empty() || !is_simple_ident(cls) || !selector_token_is_simple_type(tag) {
                return None;
            }
            return Some(SimpleSelectorPrelude::TypeClass {
                tag: tag.to_ascii_lowercase(),
                class: cls.to_string(),
            });
        }
    }
    if s.is_empty() || !selector_token_is_simple_type(s) {
        return None;
    }
    Some(SimpleSelectorPrelude::Type {
        tag: s.to_ascii_lowercase(),
    })
}

fn is_simple_ident(s: &str) -> bool {
    let mut it = s.bytes().enumerate();
    let Some((_, b0)) = it.next() else {
        return false;
    };
    if !(b0.is_ascii_alphabetic() || b0 == b'_' || b0 == b'-') {
        return false;
    }
    it.all(|(_, b)| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
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

fn subject_matches(
    subj: &SimpleSelectorPrelude,
    element_tag_lower: &str,
    classes: &[String],
    element_id: Option<&str>,
) -> bool {
    match subj {
        SimpleSelectorPrelude::Type { tag } => tag == element_tag_lower,
        SimpleSelectorPrelude::Class { name } => classes.iter().any(|c| c == name),
        SimpleSelectorPrelude::Id { name } => element_id.is_some_and(|id| id == name.as_str()),
        SimpleSelectorPrelude::IdClass { id, class } => {
            element_id.is_some_and(|eid| eid == id.as_str()) && classes.iter().any(|c| c == class)
        }
        SimpleSelectorPrelude::TypeClass { tag, class } => {
            tag == element_tag_lower && classes.iter().any(|c| c == class)
        }
        SimpleSelectorPrelude::TypeId { tag, id } => {
            tag == element_tag_lower && element_id.is_some_and(|eid| eid == id.as_str())
        }
    }
}

/// `true` when `prelude` is one ASCII type-like token equal to `element_tag_lower` (already lowercased).
pub fn prelude_matches_simple_type(prelude: &str, element_tag_lower: &str) -> bool {
    matches!(
        parse_simple_prelude(prelude),
        Some(SimpleSelectorPrelude::Type { ref tag }) if tag == element_tag_lower
    )
}

/// Cascade for one element: `element_tag_lower` is the HTML tag (`p`, `h1`, …).
///
/// `classes` are tokens from the HTML `class` attribute (case-sensitive, per HTML). `element_id`
/// is the trimmed `id` attribute when present.
pub fn cascade_element_rules(
    bundle: &[(String, Vec<ParsedQualifiedRule>)],
    element_tag_lower: &str,
    classes: &[String],
    element_id: Option<&str>,
) -> HashMap<String, String> {
    // Tie-break: specificity first, then document position (rule + declaration index).
    let mut best: HashMap<String, ((u8, u8, u32), u32, String)> = HashMap::new();
    let mut order = 0u32;
    for (_url, rules) in bundle {
        for rule in rules {
            if let Some(subj) = parse_simple_prelude(&rule.prelude_display) {
                if subject_matches(&subj, element_tag_lower, classes, element_id) {
                    let spec = specificity(&subj);
                    for (di, d) in rule.declarations.iter().enumerate() {
                        let key = d.property.to_ascii_lowercase();
                        let tie = order.saturating_mul(4096).saturating_add(di as u32);
                        let cand_val = d.value_display.clone();
                        match best.get(&key) {
                            None => {
                                best.insert(key, (spec, tie, cand_val));
                            }
                            Some((ex_spec, ex_tie, _)) => {
                                if (spec, tie) > (*ex_spec, *ex_tie) {
                                    best.insert(key, (spec, tie, cand_val));
                                }
                            }
                        }
                    }
                }
            }
            order = order.saturating_add(1);
        }
    }
    best.into_iter().map(|(k, (_, _, v))| (k, v)).collect()
}

/// For one element tag with **no** classes or id (same as [`cascade_element_rules`] with empty `class`/`id`).
pub fn cascade_simple_type_rules(
    bundle: &[(String, Vec<ParsedQualifiedRule>)],
    element_tag_lower: &str,
) -> HashMap<String, String> {
    cascade_element_rules(bundle, element_tag_lower, &[], None)
}

fn is_document_root_type_tag(tag: &str) -> bool {
    tag == "html" || tag == "body"
}

/// Declarations from rules whose prelude is the type selector `html` or `body` only.
///
/// Rules are considered in bundle order; other selectors are skipped. Among matching rules,
/// specificity is always type-level `(0,0,1)`; ties use rule order (and declaration index within
/// a rule), same as [`cascade_element_rules`].
pub fn cascade_document_defaults(
    bundle: &[(String, Vec<ParsedQualifiedRule>)],
) -> HashMap<String, String> {
    let mut best: HashMap<String, ((u8, u8, u32), u32, String)> = HashMap::new();
    let mut doc_order = 0u32;
    for (_url, rules) in bundle {
        for rule in rules {
            let Some(SimpleSelectorPrelude::Type { tag }) =
                parse_simple_prelude(&rule.prelude_display)
            else {
                continue;
            };
            if !is_document_root_type_tag(&tag) {
                continue;
            }
            let spec = (0, 0, 1);
            for (di, d) in rule.declarations.iter().enumerate() {
                let key = d.property.to_ascii_lowercase();
                let tie = doc_order.saturating_mul(4096).saturating_add(di as u32);
                let cand_val = d.value_display.clone();
                match best.get(&key) {
                    None => {
                        best.insert(key, (spec, tie, cand_val));
                    }
                    Some((ex_spec, ex_tie, _)) => {
                        if (spec, tie) > (*ex_spec, *ex_tie) {
                            best.insert(key, (spec, tie, cand_val));
                        }
                    }
                }
            }
            doc_order = doc_order.saturating_add(1);
        }
    }
    best.into_iter().map(|(k, (_, _, v))| (k, v)).collect()
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
        assert!(!prelude_matches_simple_type("p#main", "p"));
        assert!(!prelude_matches_simple_type("#main.lead", "p"));
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

    #[test]
    fn class_beats_type_by_specificity() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule("p", vec![("color", "red")]),
                rule(".lead", vec![("color", "blue")]),
            ],
        )];
        let m = cascade_element_rules(&bundle, "p", &["lead".into()], None);
        assert_eq!(m.get("color").map(String::as_str), Some("blue"));
    }

    #[test]
    fn id_beats_class() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule(".lead", vec![("color", "red")]),
                rule("#main", vec![("color", "green")]),
            ],
        )];
        let m = cascade_element_rules(&bundle, "p", &["lead".into()], Some("main"));
        assert_eq!(m.get("color").map(String::as_str), Some("green"));
    }

    #[test]
    fn type_loses_to_later_type_same_order_conflict() {
        // same specificity: second rule wins
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule("p", vec![("color", "red")]),
                rule("p", vec![("color", "navy")]),
            ],
        )];
        let m = cascade_element_rules(&bundle, "p", &["x".into()], None);
        assert_eq!(m.get("color").map(String::as_str), Some("navy"));
    }

    #[test]
    fn same_rule_duplicate_property_last_declaration_wins() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![rule("p", vec![("color", "red"), ("color", "green")])],
        )];
        let m = cascade_element_rules(&bundle, "p", &[], None);
        assert_eq!(m.get("color").map(String::as_str), Some("green"));
    }

    #[test]
    fn document_defaults_html_body_order() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule("body", vec![("color", "red")]),
                rule("p", vec![("color", "blue")]),
                rule("body", vec![("color", "green")]),
            ],
        )];
        let m = cascade_document_defaults(&bundle);
        assert_eq!(m.get("color").map(String::as_str), Some("green"));
    }

    #[test]
    fn document_defaults_skips_class_selectors() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule(".theme", vec![("color", "red")]),
                rule("html", vec![("color", "navy")]),
            ],
        )];
        let m = cascade_document_defaults(&bundle);
        assert_eq!(m.get("color").map(String::as_str), Some("navy"));
    }

    #[test]
    fn parse_type_class_prelude() {
        assert_eq!(
            parse_simple_prelude("p.lead"),
            Some(SimpleSelectorPrelude::TypeClass {
                tag: "p".into(),
                class: "lead".into(),
            })
        );
        assert_eq!(
            parse_simple_prelude("P.lead"),
            Some(SimpleSelectorPrelude::TypeClass {
                tag: "p".into(),
                class: "lead".into(),
            })
        );
        assert!(parse_simple_prelude("p.").is_none());
        assert!(parse_simple_prelude("p.lead.extra").is_none());
    }

    #[test]
    fn type_class_matches_tag_and_class() {
        assert!(subject_matches(
            &SimpleSelectorPrelude::TypeClass {
                tag: "p".into(),
                class: "lead".into(),
            },
            "p",
            &["lead".into()],
            None,
        ));
        assert!(!subject_matches(
            &SimpleSelectorPrelude::TypeClass {
                tag: "p".into(),
                class: "lead".into(),
            },
            "p",
            &[],
            None,
        ));
        assert!(!subject_matches(
            &SimpleSelectorPrelude::TypeClass {
                tag: "p".into(),
                class: "lead".into(),
            },
            "div",
            &["lead".into()],
            None,
        ));
    }

    #[test]
    fn type_class_beats_type_and_loses_to_id() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule("p", vec![("color", "red")]),
                rule("p.lead", vec![("color", "blue")]),
            ],
        )];
        let m = cascade_element_rules(&bundle, "p", &["lead".into()], None);
        assert_eq!(m.get("color").map(String::as_str), Some("blue"));

        let bundle2 = vec![(
            "https://x/a.css".into(),
            vec![
                rule("p.lead", vec![("color", "blue")]),
                rule("#x", vec![("color", "green")]),
            ],
        )];
        let m2 = cascade_element_rules(&bundle2, "p", &["lead".into()], Some("x"));
        assert_eq!(m2.get("color").map(String::as_str), Some("green"));
    }

    #[test]
    fn type_class_beats_class_alone() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule(".lead", vec![("color", "red")]),
                rule("p.lead", vec![("color", "navy")]),
            ],
        )];
        let m = cascade_element_rules(&bundle, "p", &["lead".into()], None);
        assert_eq!(m.get("color").map(String::as_str), Some("navy"));
    }

    #[test]
    fn parse_type_id_prelude() {
        assert_eq!(
            parse_simple_prelude("p#main"),
            Some(SimpleSelectorPrelude::TypeId {
                tag: "p".into(),
                id: "main".into(),
            })
        );
        assert_eq!(
            parse_simple_prelude("P#main"),
            Some(SimpleSelectorPrelude::TypeId {
                tag: "p".into(),
                id: "main".into(),
            })
        );
        assert!(parse_simple_prelude("p#").is_none());
        assert!(parse_simple_prelude("p#main#y").is_none());
    }

    #[test]
    fn type_id_matches_tag_and_id() {
        assert!(subject_matches(
            &SimpleSelectorPrelude::TypeId {
                tag: "p".into(),
                id: "x".into(),
            },
            "p",
            &[],
            Some("x"),
        ));
        assert!(!subject_matches(
            &SimpleSelectorPrelude::TypeId {
                tag: "p".into(),
                id: "x".into(),
            },
            "p",
            &[],
            None,
        ));
        assert!(!subject_matches(
            &SimpleSelectorPrelude::TypeId {
                tag: "p".into(),
                id: "x".into(),
            },
            "div",
            &[],
            Some("x"),
        ));
    }

    #[test]
    fn type_id_beats_plain_id_and_type_class() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule("#main", vec![("color", "red")]),
                rule("p#main", vec![("color", "green")]),
            ],
        )];
        let m = cascade_element_rules(&bundle, "p", &[], Some("main"));
        assert_eq!(m.get("color").map(String::as_str), Some("green"));

        let bundle2 = vec![(
            "https://x/a.css".into(),
            vec![
                rule("p.lead", vec![("color", "blue")]),
                rule("p#main", vec![("color", "navy")]),
            ],
        )];
        let m2 = cascade_element_rules(&bundle2, "p", &["lead".into()], Some("main"));
        assert_eq!(m2.get("color").map(String::as_str), Some("navy"));
    }

    #[test]
    fn parse_id_class_prelude() {
        assert_eq!(
            parse_simple_prelude("#main.lead"),
            Some(SimpleSelectorPrelude::IdClass {
                id: "main".into(),
                class: "lead".into(),
            })
        );
        assert!(parse_simple_prelude("#.").is_none());
        assert!(parse_simple_prelude("#main.").is_none());
        assert!(parse_simple_prelude("#main.lead.extra").is_none());
    }

    #[test]
    fn id_class_matches_id_and_class() {
        assert!(subject_matches(
            &SimpleSelectorPrelude::IdClass {
                id: "x".into(),
                class: "lead".into(),
            },
            "p",
            &["lead".into()],
            Some("x"),
        ));
        assert!(!subject_matches(
            &SimpleSelectorPrelude::IdClass {
                id: "x".into(),
                class: "lead".into(),
            },
            "p",
            &[],
            Some("x"),
        ));
        assert!(!subject_matches(
            &SimpleSelectorPrelude::IdClass {
                id: "x".into(),
                class: "lead".into(),
            },
            "p",
            &["lead".into()],
            Some("y"),
        ));
    }

    #[test]
    fn id_class_beats_type_id_and_plain_id() {
        let bundle = vec![(
            "https://x/a.css".into(),
            vec![
                rule("p#main", vec![("color", "green")]),
                rule("#main.lead", vec![("color", "navy")]),
            ],
        )];
        let m = cascade_element_rules(&bundle, "p", &["lead".into()], Some("main"));
        assert_eq!(m.get("color").map(String::as_str), Some("navy"));

        let bundle2 = vec![(
            "https://x/a.css".into(),
            vec![
                rule("#main", vec![("color", "red")]),
                rule("#main.lead", vec![("color", "blue")]),
            ],
        )];
        let m2 = cascade_element_rules(&bundle2, "p", &["lead".into()], Some("main"));
        assert_eq!(m2.get("color").map(String::as_str), Some("blue"));
    }
}
