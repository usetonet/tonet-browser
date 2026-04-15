//! Map fetched author CSS (`ParsedQualifiedRule` bundles) into egui paint hints (`color`, `font-size`).

use egui::Color32;
use tonet_engine::css::{cascade_element_rules, ParsedQualifiedRule};

use crate::parser::DomNode;

/// Per-node overrides from author stylesheets (simple type / class / id selectors).
#[derive(Debug, Clone, Copy, Default)]
pub struct DomNodePaintHints {
    pub color: Option<Color32>,
    pub font_size: Option<f32>,
}

fn trim_css_ascii_whitespace(s: &str) -> &str {
    s.trim_matches(|c: char| matches!(c, ' ' | '\t' | '\n' | '\r' | '\x0c'))
}

/// Parse `color` values: `#rgb`, `#rrggbb`, `rgb()`, `rgba()` with **integer** components, and a small named set.
pub fn parse_css_color(value: &str) -> Option<Color32> {
    let s = trim_css_ascii_whitespace(value);
    if s.is_empty() {
        return None;
    }
    if s.eq_ignore_ascii_case("transparent") {
        return Some(Color32::TRANSPARENT);
    }
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex_color(hex);
    }
    let lower = s.to_ascii_lowercase();
    if lower.starts_with("rgb(") || lower.starts_with("rgba(") {
        return parse_rgb_function(s);
    }
    named_color(s)
}

fn parse_hex_color(hex: &str) -> Option<Color32> {
    let h = hex.trim();
    match h.len() {
        3 => {
            let mut v = [0u8; 3];
            for (i, ch) in h.bytes().enumerate() {
                let n = hex_nibble(ch)?;
                v[i] = n * 17;
            }
            Some(Color32::from_rgb(v[0], v[1], v[2]))
        }
        6 => {
            let r = u8::from_str_radix(&h[0..2], 16).ok()?;
            let g = u8::from_str_radix(&h[2..4], 16).ok()?;
            let b = u8::from_str_radix(&h[4..6], 16).ok()?;
            Some(Color32::from_rgb(r, g, b))
        }
        _ => None,
    }
}

fn hex_nibble(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(10 + (b - b'a')),
        b'A'..=b'F' => Some(10 + (b - b'A')),
        _ => None,
    }
}

fn parse_rgb_function(s: &str) -> Option<Color32> {
    let open = s.find('(')?;
    let inner = s[open + 1..].strip_suffix(')')?.trim();
    let mut parts = inner.split(',');
    let r: u8 = parts.next()?.trim().parse().ok()?;
    let g: u8 = parts.next()?.trim().parse().ok()?;
    let b: u8 = parts.next()?.trim().parse().ok()?;
    let a = if let Some(fourth) = parts.next() {
        let t = fourth.trim();
        if let Ok(ai) = t.parse::<u8>() {
            ai
        } else {
            255
        }
    } else {
        255
    };
    Some(Color32::from_rgba_unmultiplied(r, g, b, a))
}

fn named_color(s: &str) -> Option<Color32> {
    Some(match s.to_ascii_lowercase().as_str() {
        "black" => Color32::BLACK,
        "white" => Color32::WHITE,
        "red" => Color32::RED,
        "green" => Color32::GREEN,
        "blue" => Color32::BLUE,
        "navy" => Color32::from_rgb(0, 0, 128),
        "maroon" => Color32::from_rgb(128, 0, 0),
        "gray" | "grey" => Color32::from_rgb(128, 128, 128),
        "silver" => Color32::from_rgb(192, 192, 192),
        "yellow" => Color32::YELLOW,
        "olive" => Color32::from_rgb(128, 128, 0),
        "lime" => Color32::from_rgb(0, 255, 0),
        "aqua" | "cyan" => Color32::from_rgb(0, 255, 255),
        "teal" => Color32::from_rgb(0, 128, 128),
        "purple" => Color32::from_rgb(128, 0, 128),
        "fuchsia" | "magenta" => Color32::from_rgb(255, 0, 255),
        "orange" => Color32::from_rgb(255, 165, 0),
        _ => return None,
    })
}

/// Parse `font-size`: `px`, `em`, `rem`, `%` against `root_px` (used as root em and 100% base).
pub fn parse_font_size_px(value: &str, root_px: f32) -> Option<f32> {
    let s = trim_css_ascii_whitespace(value).to_ascii_lowercase();
    if s.ends_with("px") {
        let n: f32 = s[..s.len() - 2].trim().parse().ok()?;
        return Some(n);
    }
    if s.ends_with("rem") || s.ends_with("em") {
        let end = if s.ends_with("rem") {
            s.len() - 3
        } else {
            s.len() - 2
        };
        let n: f32 = s[..end].trim().parse().ok()?;
        return Some(n * root_px);
    }
    if s.ends_with('%') {
        let n: f32 = s[..s.len() - 1].trim().parse().ok()?;
        return Some(n / 100.0 * root_px);
    }
    None
}

fn clamp_font(px: f32) -> f32 {
    px.clamp(6.0, 256.0)
}

/// Build one [`DomNodePaintHints`] per DOM node from `bundle` (same order as `nodes`).
pub fn compute_dom_paint_hints(
    nodes: &[DomNode],
    bundle: &[(String, Vec<ParsedQualifiedRule>)],
) -> Vec<DomNodePaintHints> {
    const ROOT_PX: f32 = 16.0;
    nodes
        .iter()
        .map(|n| {
            let m = cascade_element_rules(
                bundle,
                n.kind.tag_name(),
                &n.classes,
                n.element_id.as_deref(),
            );
            let color = m.get("color").and_then(|v| parse_css_color(v));
            let font_size = m
                .get("font-size")
                .and_then(|v| parse_font_size_px(v, ROOT_PX))
                .map(clamp_font);
            DomNodePaintHints { color, font_size }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{DomNode, DomNodeType};
    use tonet_engine::css::{ParsedQualifiedRule, SimpleDeclaration};

    #[test]
    fn hex_six() {
        let c = parse_css_color("#00aabb").unwrap();
        assert_eq!(c.r(), 0);
        assert_eq!(c.g(), 170);
        assert_eq!(c.b(), 187);
    }

    #[test]
    fn rgb_triple() {
        let c = parse_css_color("rgb(10, 20, 30)").unwrap();
        assert_eq!((c.r(), c.g(), c.b()), (10, 20, 30));
    }

    #[test]
    fn font_em() {
        let px = parse_font_size_px("1.5em", 16.0).unwrap();
        assert!((px - 24.0).abs() < 0.01);
    }

    #[test]
    fn compute_hints_for_paragraph_color() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "Hi".into(),
            href: None,
            classes: Vec::new(),
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![ParsedQualifiedRule {
                prelude_display: "p".into(),
                declarations: vec![SimpleDeclaration {
                    property: "color".into(),
                    value_display: "navy".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints.len(), 1);
        assert!(hints[0].font_size.is_none());
        let c = hints[0].color.expect("color");
        assert_eq!((c.r(), c.g(), c.b()), (0, 0, 128));
    }

    #[test]
    fn class_selector_overrides_type() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "Hi".into(),
            href: None,
            classes: vec!["lead".into()],
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![
                ParsedQualifiedRule {
                    prelude_display: "p".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "color".into(),
                        value_display: "red".into(),
                    }],
                },
                ParsedQualifiedRule {
                    prelude_display: ".lead".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "color".into(),
                        value_display: "blue".into(),
                    }],
                },
            ],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].color, Some(Color32::BLUE));
    }
}
