//! Map fetched author CSS (`ParsedQualifiedRule` bundles) into egui paint hints (`color`, `font-size`,
//! `line-height`, `letter-spacing`, `font-weight`, `font-style`, `margin` / `margin-top` / `margin-bottom`, `text-decoration`, `text-align`, `text-transform`).

use std::borrow::Cow;
use std::collections::HashMap;

use egui::Color32;
use tonet_engine::css::{cascade_document_defaults, cascade_element_rules, ParsedQualifiedRule};

use crate::parser::{DomNode, DomNodeType};

/// Subset of CSS `text-align` for the LTR read view (`start`/`end` ≈ left/right).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignHint {
    #[default]
    Start,
    Center,
    End,
}

/// Parse `text-align` keywords we support (`justify` → unsupported / `None`).
pub fn parse_text_align(value: &str) -> Option<TextAlignHint> {
    let t = trim_css_ascii_whitespace(value).to_ascii_lowercase();
    match t.as_str() {
        "left" | "start" => Some(TextAlignHint::Start),
        "center" => Some(TextAlignHint::Center),
        "right" | "end" => Some(TextAlignHint::End),
        "justify" => None,
        _ => None,
    }
}

/// Subset of CSS `text-transform` for the read view (`full-width` / etc. → unsupported).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextTransformHint {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

pub fn parse_text_transform(value: &str) -> Option<TextTransformHint> {
    match trim_css_ascii_whitespace(value).to_ascii_lowercase().as_str() {
        "none" => Some(TextTransformHint::None),
        "uppercase" => Some(TextTransformHint::Uppercase),
        "lowercase" => Some(TextTransformHint::Lowercase),
        "capitalize" => Some(TextTransformHint::Capitalize),
        _ => None,
    }
}

fn capitalize_words_css(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut at_word_start = true;
    for ch in s.chars() {
        if ch.is_whitespace() {
            out.push(ch);
            at_word_start = true;
        } else if at_word_start {
            out.extend(ch.to_uppercase());
            at_word_start = false;
        } else {
            out.extend(ch.to_lowercase());
        }
    }
    out
}

/// Apply [`TextTransformHint`] to raw node text for display (merged from author CSS).
pub fn display_text_cow<'a>(raw: &'a str, transform: Option<TextTransformHint>) -> Cow<'a, str> {
    match transform {
        None | Some(TextTransformHint::None) => Cow::Borrowed(raw),
        Some(TextTransformHint::Uppercase) => Cow::Owned(raw.to_uppercase()),
        Some(TextTransformHint::Lowercase) => Cow::Owned(raw.to_lowercase()),
        Some(TextTransformHint::Capitalize) => Cow::Owned(capitalize_words_css(raw)),
    }
}

/// Per-node overrides from author stylesheets (simple type / class / id selectors).
#[derive(Debug, Clone, Copy, Default)]
pub struct DomNodePaintHints {
    pub color: Option<Color32>,
    pub font_size: Option<f32>,
    /// Resolved line height in **points** for egui (`RichText::line_height`); `None` = CSS `normal` / default.
    pub line_height_px: Option<f32>,
    /// Extra tracking in **points** (`RichText::extra_letter_spacing`); `None` = CSS `normal` / default.
    pub letter_spacing_px: Option<f32>,
    /// Resolved CSS weight (`100`…`900`) when `font-weight` was set.
    pub font_weight: Option<u16>,
    /// When `Some(true)` / `Some(false)`, mirrors author `font-style` italic vs normal.
    pub font_style_italic: Option<bool>,
    /// Vertical spacing before the block (`margin-top`). **Not** inherited from `html`/`body`.
    pub margin_top: Option<f32>,
    /// Vertical spacing after the block (`margin-bottom`). **Not** inherited from `html`/`body`.
    pub margin_bottom: Option<f32>,
    /// `text-decoration` underline (`Some(true)` / `Some(false)`); merged with `html`/`body` like typography.
    pub underline: Option<bool>,
    /// Horizontal alignment for the node’s text line(s); merged with `html`/`body`.
    pub text_align: Option<TextAlignHint>,
    /// Casing of displayed text; merged with `html`/`body`.
    pub text_transform: Option<TextTransformHint>,
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

fn clamp_line_height(px: f32) -> f32 {
    px.clamp(4.0, 400.0)
}

/// Default `font-size` in px when the author did not set one — must match [`crate::renderer`] defaults.
fn default_font_size_px(kind: DomNodeType) -> f32 {
    match kind {
        DomNodeType::Title => 21.0,
        DomNodeType::H1 => 26.0,
        DomNodeType::H2 => 19.0,
        DomNodeType::Paragraph | DomNodeType::Link => 15.0,
    }
}

/// Parse `line-height` against the node’s **used** `font-size` and root px (`rem` / `%` base).
///
/// `normal` yields `None` (egui picks default). Unitless numbers multiply `font_size_px`.
pub fn parse_line_height(value: &str, font_size_px: f32, root_px: f32) -> Option<f32> {
    let s = trim_css_ascii_whitespace(value);
    if s.is_empty() {
        return None;
    }
    let lower = s.to_ascii_lowercase();
    if lower == "normal" {
        return None;
    }
    if lower.ends_with("px") {
        let n: f32 = lower[..lower.len() - 2].trim().parse().ok()?;
        return (n.is_finite() && n > 0.0).then_some(clamp_line_height(n));
    }
    if lower.ends_with("rem") {
        let n: f32 = lower[..lower.len() - 3].trim().parse().ok()?;
        return (n.is_finite() && n > 0.0).then_some(clamp_line_height(n * root_px));
    }
    if lower.ends_with("em") {
        let n: f32 = lower[..lower.len() - 2].trim().parse().ok()?;
        return (n.is_finite() && n > 0.0).then_some(clamp_line_height(n * font_size_px));
    }
    if lower.ends_with('%') {
        let n: f32 = lower[..lower.len() - 1].trim().parse().ok()?;
        return (n.is_finite() && n > 0.0).then_some(clamp_line_height(n / 100.0 * font_size_px));
    }
    let n: f32 = lower.parse().ok()?;
    (n.is_finite() && n > 0.0).then_some(clamp_line_height(n * font_size_px))
}

fn clamp_letter_spacing(px: f32) -> f32 {
    px.clamp(-24.0, 48.0)
}

/// Parse `letter-spacing`: `normal` → `None`; `px` / `em` / `rem` (negative allowed).
pub fn parse_letter_spacing(value: &str, font_size_px: f32, root_px: f32) -> Option<f32> {
    let s = trim_css_ascii_whitespace(value);
    if s.is_empty() {
        return None;
    }
    let lower = s.to_ascii_lowercase();
    if lower == "normal" {
        return None;
    }
    if lower.ends_with("px") {
        let n: f32 = lower[..lower.len() - 2].trim().parse().ok()?;
        return n.is_finite().then_some(clamp_letter_spacing(n));
    }
    if lower.ends_with("rem") {
        let n: f32 = lower[..lower.len() - 3].trim().parse().ok()?;
        return n.is_finite().then_some(clamp_letter_spacing(n * root_px));
    }
    if lower.ends_with("em") {
        let n: f32 = lower[..lower.len() - 2].trim().parse().ok()?;
        return n.is_finite().then_some(clamp_letter_spacing(n * font_size_px));
    }
    None
}

fn clamp_margin(px: f32) -> f32 {
    px.clamp(0.0, 240.0)
}

/// `text-decoration`: `underline` → `true`, `none` → `false`, else `None` (unsupported tokens ignored unless `underline` appears).
pub fn parse_text_decoration_underline(value: &str) -> Option<bool> {
    let t = trim_css_ascii_whitespace(value).to_ascii_lowercase();
    if t.is_empty() {
        return None;
    }
    if t == "none" {
        return Some(false);
    }
    if t.split_whitespace().any(|tok| tok == "underline") {
        return Some(true);
    }
    None
}

/// Expand CSS `margin` shorthand into `(top, right, bottom, left)` using the same length parser as `font-size`.
fn parse_margin_shorthand_lengths(
    value: &str,
    root_px: f32,
) -> (Option<f32>, Option<f32>, Option<f32>, Option<f32>) {
    let tokens: Vec<&str> = trim_css_ascii_whitespace(value)
        .split_whitespace()
        .collect();
    let p = |t: &str| parse_font_size_px(t, root_px).map(clamp_margin);
    match tokens.len() {
        0 => (None, None, None, None),
        1 => {
            let v = p(tokens[0]);
            (v, v, v, v)
        }
        2 => {
            let tb = p(tokens[0]);
            let rl = p(tokens[1]);
            (tb, rl, tb, rl)
        }
        3 => {
            let t = p(tokens[0]);
            let r = p(tokens[1]);
            let b = p(tokens[2]);
            let l = r;
            (t, r, b, l)
        }
        _ => {
            let t = p(tokens[0]);
            let r = tokens.get(1).and_then(|x| p(x));
            let b = tokens.get(2).and_then(|x| p(x));
            let l = tokens.get(3).and_then(|x| p(x));
            (t, r, b, l)
        }
    }
}

fn resolve_margin_top(m: &HashMap<String, String>, root_px: f32) -> Option<f32> {
    if let Some(v) = m.get("margin-top") {
        return parse_font_size_px(v, root_px).map(clamp_margin);
    }
    m.get("margin")
        .and_then(|sh| parse_margin_shorthand_lengths(sh, root_px).0)
}

fn resolve_margin_bottom(m: &HashMap<String, String>, root_px: f32) -> Option<f32> {
    if let Some(v) = m.get("margin-bottom") {
        return parse_font_size_px(v, root_px).map(clamp_margin);
    }
    m.get("margin")
        .and_then(|sh| parse_margin_shorthand_lengths(sh, root_px).2)
}

/// Parse `font-weight` keywords and numeric `100`…`900`.
pub fn parse_font_weight(value: &str) -> Option<u16> {
    let s = trim_css_ascii_whitespace(value);
    if s.is_empty() {
        return None;
    }
    let lower = s.to_ascii_lowercase();
    match lower.as_str() {
        "normal" => Some(400),
        "bold" | "bolder" => Some(700),
        "lighter" => Some(300),
        _ => {
            let n: f32 = s.parse().ok()?;
            if !n.is_finite() || n <= 0.0 {
                return None;
            }
            let w = n.round() as u16;
            Some(w.clamp(100, 900))
        }
    }
}

/// Parse `font-style`: italic / oblique → `true`, normal → `false`.
pub fn parse_font_style(value: &str) -> Option<bool> {
    let t = trim_css_ascii_whitespace(value).to_ascii_lowercase();
    match t.as_str() {
        "italic" | "oblique" => Some(true),
        "normal" => Some(false),
        _ => None,
    }
}

fn merged_author_value<'a>(
    node: &'a HashMap<String, String>,
    doc: &'a HashMap<String, String>,
    key: &str,
) -> Option<&'a str> {
    node.get(key).or(doc.get(key)).map(String::as_str)
}

/// Build one [`DomNodePaintHints`] per DOM node from `bundle` (same order as `nodes`).
///
/// Declarations from `html` / `body` **type** rules (`tonet_engine::css::cascade_document_defaults`)
/// apply when a node does not set the same property.
pub fn compute_dom_paint_hints(
    nodes: &[DomNode],
    bundle: &[(String, Vec<ParsedQualifiedRule>)],
) -> Vec<DomNodePaintHints> {
    const ROOT_PX: f32 = 16.0;
    let doc = cascade_document_defaults(bundle);
    nodes
        .iter()
        .map(|n| {
            let m = cascade_element_rules(
                bundle,
                n.kind.tag_name(),
                &n.classes,
                n.element_id.as_deref(),
            );
            let color = merged_author_value(&m, &doc, "color").and_then(parse_css_color);
            let font_size = merged_author_value(&m, &doc, "font-size")
                .and_then(|v| parse_font_size_px(v, ROOT_PX))
                .map(clamp_font);
            let used_font_size = font_size.unwrap_or_else(|| default_font_size_px(n.kind));
            let line_height_px = merged_author_value(&m, &doc, "line-height")
                .and_then(|v| parse_line_height(v, used_font_size, ROOT_PX));
            let letter_spacing_px = merged_author_value(&m, &doc, "letter-spacing")
                .and_then(|v| parse_letter_spacing(v, used_font_size, ROOT_PX));
            let font_weight =
                merged_author_value(&m, &doc, "font-weight").and_then(parse_font_weight);
            let font_style_italic =
                merged_author_value(&m, &doc, "font-style").and_then(parse_font_style);
            // Margins are not inherited; longhands win over `margin` shorthand when present.
            let margin_top = resolve_margin_top(&m, ROOT_PX);
            let margin_bottom = resolve_margin_bottom(&m, ROOT_PX);
            let underline = merged_author_value(&m, &doc, "text-decoration")
                .and_then(parse_text_decoration_underline);
            let text_align = merged_author_value(&m, &doc, "text-align").and_then(parse_text_align);
            let text_transform =
                merged_author_value(&m, &doc, "text-transform").and_then(parse_text_transform);
            DomNodePaintHints {
                color,
                font_size,
                line_height_px,
                letter_spacing_px,
                font_weight,
                font_style_italic,
                margin_top,
                margin_bottom,
                underline,
                text_align,
                text_transform,
            }
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
    fn font_weight_bold_keyword() {
        assert_eq!(parse_font_weight("bold"), Some(700));
        assert_eq!(parse_font_weight("normal"), Some(400));
        assert_eq!(parse_font_weight("600"), Some(600));
    }

    #[test]
    fn font_style_italic() {
        assert_eq!(parse_font_style("italic"), Some(true));
        assert_eq!(parse_font_style("normal"), Some(false));
    }

    #[test]
    fn hints_font_weight_on_heading() {
        let nodes = vec![DomNode {
            kind: DomNodeType::H1,
            text: "T".into(),
            href: None,
            classes: Vec::new(),
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![ParsedQualifiedRule {
                prelude_display: "h1".into(),
                declarations: vec![SimpleDeclaration {
                    property: "font-weight".into(),
                    value_display: "400".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].font_weight, Some(400));
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

    #[test]
    fn paragraph_inherits_body_color() {
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
                prelude_display: "body".into(),
                declarations: vec![SimpleDeclaration {
                    property: "color".into(),
                    value_display: "green".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].color, Some(Color32::GREEN));
    }

    #[test]
    fn margin_top_on_paragraph() {
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
                    property: "margin-top".into(),
                    value_display: "12px".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].margin_top, Some(12.0));
        assert!(hints[0].margin_bottom.is_none());
    }

    #[test]
    fn margin_not_inherited_from_body() {
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
                prelude_display: "body".into(),
                declarations: vec![SimpleDeclaration {
                    property: "margin-top".into(),
                    value_display: "40px".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert!(hints[0].margin_top.is_none());
    }

    #[test]
    fn margin_shorthand_sets_top_and_bottom() {
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
                    property: "margin".into(),
                    value_display: "8px".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].margin_top, Some(8.0));
        assert_eq!(hints[0].margin_bottom, Some(8.0));
    }

    #[test]
    fn margin_top_longhand_overrides_shorthand() {
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
                declarations: vec![
                    SimpleDeclaration {
                        property: "margin".into(),
                        value_display: "1px".into(),
                    },
                    SimpleDeclaration {
                        property: "margin-top".into(),
                        value_display: "20px".into(),
                    },
                ],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].margin_top, Some(20.0));
        assert_eq!(hints[0].margin_bottom, Some(1.0));
    }

    #[test]
    fn text_decoration_underline_inherited_from_body() {
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
                prelude_display: "body".into(),
                declarations: vec![SimpleDeclaration {
                    property: "text-decoration".into(),
                    value_display: "underline".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].underline, Some(true));
    }

    #[test]
    fn parse_text_decoration_keywords() {
        assert_eq!(parse_text_decoration_underline("none"), Some(false));
        assert_eq!(parse_text_decoration_underline("underline"), Some(true));
        assert_eq!(
            parse_text_decoration_underline("underline dotted"),
            Some(true)
        );
    }

    #[test]
    fn parse_text_align_keywords() {
        assert_eq!(parse_text_align("left"), Some(TextAlignHint::Start));
        assert_eq!(parse_text_align("START"), Some(TextAlignHint::Start));
        assert_eq!(parse_text_align("center"), Some(TextAlignHint::Center));
        assert_eq!(parse_text_align("right"), Some(TextAlignHint::End));
        assert_eq!(parse_text_align("end"), Some(TextAlignHint::End));
        assert_eq!(parse_text_align("justify"), None);
    }

    #[test]
    fn text_align_inherited_from_body() {
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
                prelude_display: "body".into(),
                declarations: vec![SimpleDeclaration {
                    property: "text-align".into(),
                    value_display: "center".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].text_align, Some(TextAlignHint::Center));
    }

    #[test]
    fn parse_line_height_normal_and_unitless() {
        assert_eq!(parse_line_height("normal", 15.0, 16.0), None);
        assert_eq!(parse_line_height("1.5", 20.0, 16.0), Some(30.0));
        assert_eq!(parse_line_height("24px", 15.0, 16.0), Some(24.0));
        assert_eq!(parse_line_height("150%", 20.0, 16.0), Some(30.0));
        assert_eq!(parse_line_height("2em", 10.0, 16.0), Some(20.0));
        assert!((parse_line_height("1.25rem", 15.0, 16.0).unwrap() - 20.0).abs() < 0.01);
    }

    #[test]
    fn line_height_inherited_from_body() {
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
                prelude_display: "body".into(),
                declarations: vec![SimpleDeclaration {
                    property: "line-height".into(),
                    value_display: "2".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].line_height_px, Some(30.0));
    }

    #[test]
    fn line_height_unitless_uses_author_font_size() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "Hi".into(),
            href: None,
            classes: Vec::new(),
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![
                ParsedQualifiedRule {
                    prelude_display: "body".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "line-height".into(),
                        value_display: "2".into(),
                    }],
                },
                ParsedQualifiedRule {
                    prelude_display: "p".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "font-size".into(),
                        value_display: "20px".into(),
                    }],
                },
            ],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].font_size, Some(20.0));
        assert_eq!(hints[0].line_height_px, Some(40.0));
    }

    #[test]
    fn parse_letter_spacing_normal_px_em() {
        assert_eq!(parse_letter_spacing("normal", 15.0, 16.0), None);
        assert_eq!(parse_letter_spacing("2px", 15.0, 16.0), Some(2.0));
        assert_eq!(parse_letter_spacing("-1px", 15.0, 16.0), Some(-1.0));
        assert_eq!(parse_letter_spacing("0.1em", 20.0, 16.0), Some(2.0));
        assert!((parse_letter_spacing("0.25rem", 15.0, 16.0).unwrap() - 4.0).abs() < 0.01);
    }

    #[test]
    fn letter_spacing_inherited_from_body() {
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
                prelude_display: "body".into(),
                declarations: vec![SimpleDeclaration {
                    property: "letter-spacing".into(),
                    value_display: "3px".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].letter_spacing_px, Some(3.0));
    }

    #[test]
    fn parse_text_transform_keywords() {
        assert_eq!(parse_text_transform("none"), Some(TextTransformHint::None));
        assert_eq!(
            parse_text_transform("UPPERCASE"),
            Some(TextTransformHint::Uppercase)
        );
        assert_eq!(parse_text_transform("full-width"), None);
    }

    #[test]
    fn display_text_capitalize_words() {
        assert_eq!(
            display_text_cow("hello WORLD", Some(TextTransformHint::Capitalize)).as_ref(),
            "Hello World"
        );
        assert_eq!(
            display_text_cow("Hi", Some(TextTransformHint::Uppercase)).as_ref(),
            "HI"
        );
        assert_eq!(
            display_text_cow("NONE", Some(TextTransformHint::None)).as_ref(),
            "NONE"
        );
    }

    #[test]
    fn text_transform_inherited_from_body() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "abc".into(),
            href: None,
            classes: Vec::new(),
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![ParsedQualifiedRule {
                prelude_display: "body".into(),
                declarations: vec![SimpleDeclaration {
                    property: "text-transform".into(),
                    value_display: "uppercase".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].text_transform, Some(TextTransformHint::Uppercase));
    }

    #[test]
    fn text_transform_none_on_element_overrides_body() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "abc".into(),
            href: None,
            classes: Vec::new(),
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![
                ParsedQualifiedRule {
                    prelude_display: "body".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "text-transform".into(),
                        value_display: "uppercase".into(),
                    }],
                },
                ParsedQualifiedRule {
                    prelude_display: "p".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "text-transform".into(),
                        value_display: "none".into(),
                    }],
                },
            ],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].text_transform, Some(TextTransformHint::None));
    }
}
