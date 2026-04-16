//! Map fetched author CSS (`ParsedQualifiedRule` bundles) into egui paint hints (`color`, `font-size`,
//! `line-height`, `letter-spacing`, `font-weight`, `font-style`, `margin` / `margin-top` / `margin-bottom`, `text-decoration`, `text-align`, `text-transform`, `text-indent`, `opacity`, `visibility`, `display`, `white-space`, `word-break`, `overflow-wrap` / `word-wrap`, `max-width`, `padding` shorthand, `padding-left` / `padding-right`).

use std::borrow::Cow;
use std::collections::HashMap;

use egui::Color32;
use tonet_engine::css::{cascade_document_defaults, cascade_element_rules, ParsedQualifiedRule};

use crate::parser::{DomNode, DomNodeType};

/// Root em / `rem` base (px) for author length resolution in [`compute_dom_paint_hints`].
pub const AUTHOR_STYLE_ROOT_PX: f32 = 16.0;

/// Subset of CSS `text-align` for the LTR read view (`start`/`end` ≈ left/right).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignHint {
    #[default]
    Start,
    Center,
    End,
}

/// CSS `visibility` in the read view. `collapse` is treated like `hidden` until table layout exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VisibilityHint {
    #[default]
    Visible,
    Hidden,
}

pub fn parse_visibility(value: &str) -> Option<VisibilityHint> {
    match trim_css_ascii_whitespace(value).to_ascii_lowercase().as_str() {
        "visible" => Some(VisibilityHint::Visible),
        "hidden" | "collapse" => Some(VisibilityHint::Hidden),
        _ => None,
    }
}

/// Subset of CSS `display` for the flat read view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayHint {
    /// `block`, `inline`, `flex`, etc. — drawn like other blocks.
    Flow,
    /// `display: none` — omit the node (no margins); unlike [`VisibilityHint::Hidden`], no layout space.
    None,
}

/// Parse `display` for read layout. Only `none` is stored as [`DisplayHint::None`]; other keywords map to [`DisplayHint::Flow`] when recognized.
pub fn parse_display(value: &str) -> Option<DisplayHint> {
    let t = trim_css_ascii_whitespace(value).to_ascii_lowercase();
    match t.as_str() {
        "none" => Some(DisplayHint::None),
        "block"
        | "inline"
        | "inline-block"
        | "flow-root"
        | "flex"
        | "inline-flex"
        | "grid"
        | "inline-grid"
        | "table"
        | "table-row"
        | "table-cell"
        | "list-item"
        | "contents" => Some(DisplayHint::Flow),
        _ => None,
    }
}

/// Subset of CSS `white-space` for read-layout text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhiteSpaceHint {
    #[default]
    Normal,
    /// `nowrap` — single row until `\n`; no soft wrap at read-area width.
    Nowrap,
}

/// Parse `white-space`. `pre` / `break-spaces` → `None` (not implemented); `pre-wrap` / `pre-line` treated like `normal` (soft wrap).
pub fn parse_white_space(value: &str) -> Option<WhiteSpaceHint> {
    match trim_css_ascii_whitespace(value).to_ascii_lowercase().as_str() {
        "nowrap" => Some(WhiteSpaceHint::Nowrap),
        "normal" | "pre-wrap" | "pre-line" => Some(WhiteSpaceHint::Normal),
        _ => None,
    }
}

/// Subset of CSS `word-break` for soft-wrapped read text (egui `TextWrapping::break_anywhere`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordBreakHint {
    #[default]
    Normal,
    BreakAll,
}

pub fn parse_word_break(value: &str) -> Option<WordBreakHint> {
    match trim_css_ascii_whitespace(value).to_ascii_lowercase().as_str() {
        "normal" => Some(WordBreakHint::Normal),
        "break-all" => Some(WordBreakHint::BreakAll),
        _ => None,
    }
}

/// Subset of CSS `overflow-wrap` / legacy `word-wrap` for soft-wrapped lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OverflowWrapHint {
    #[default]
    Normal,
    Anywhere,
    BreakWord,
}

pub fn parse_overflow_wrap(value: &str) -> Option<OverflowWrapHint> {
    match trim_css_ascii_whitespace(value).to_ascii_lowercase().as_str() {
        "normal" => Some(OverflowWrapHint::Normal),
        "anywhere" => Some(OverflowWrapHint::Anywhere),
        "break-word" => Some(OverflowWrapHint::BreakWord),
        _ => None,
    }
}

/// Merge `overflow-wrap` with legacy `word-wrap` (same cascade maps). If both keys exist, `overflow-wrap` wins.
fn merged_overflow_wrap<'a>(
    node: &'a HashMap<String, String>,
    doc: &'a HashMap<String, String>,
) -> Option<&'a str> {
    merged_author_value(node, doc, "overflow-wrap")
        .or_else(|| merged_author_value(node, doc, "word-wrap"))
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
///
/// `capitalize` follows CSS Text Level 3: UAX \#29 word boundaries (`split_word_bounds`), extended
/// grapheme clusters, and only the first grapheme cluster of each word that begins with an
/// alphabetic character is case-mapped; the rest of that word is unchanged (not lowercased).
/// `uppercase` / `lowercase` use Unicode full case mappings from the standard library.
/// Locale-sensitive casing (e.g. Turkish `i` / `İ`) waits on a reliable document `lang` (or
/// equivalent) in the read path.
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

/// Parsed `text-indent` length; [`resolve_text_indent_px`] needs used `font-size`, root px,
/// and **line width** (containing block width at paint time) for `%`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextIndentSpec {
    Px(f32),
    Em(f32),
    Rem(f32),
    Percent(f32),
}

pub fn parse_text_indent(value: &str) -> Option<TextIndentSpec> {
    let s = trim_css_ascii_whitespace(value).to_ascii_lowercase();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_suffix("px") {
        let n: f32 = rest.trim().parse().ok()?;
        return n.is_finite().then_some(TextIndentSpec::Px(n));
    }
    if s.ends_with("rem") {
        let n: f32 = s[..s.len() - 3].trim().parse().ok()?;
        return n.is_finite().then_some(TextIndentSpec::Rem(n));
    }
    if s.ends_with("em") {
        let n: f32 = s[..s.len() - 2].trim().parse().ok()?;
        return n.is_finite().then_some(TextIndentSpec::Em(n));
    }
    if s.ends_with('%') {
        let n: f32 = s[..s.len() - 1].trim().parse().ok()?;
        return n.is_finite().then_some(TextIndentSpec::Percent(n));
    }
    None
}

/// Resolve [`TextIndentSpec`] to pixels for layout (`%` uses `line_width_px` as the containing width).
pub fn resolve_text_indent_px(
    spec: TextIndentSpec,
    used_font_size: f32,
    root_px: f32,
    line_width_px: f32,
) -> f32 {
    let w = line_width_px.max(1.0);
    let raw = match spec {
        TextIndentSpec::Px(v) => v,
        TextIndentSpec::Em(v) => v * used_font_size,
        TextIndentSpec::Rem(v) => v * root_px,
        TextIndentSpec::Percent(p) => p / 100.0 * w,
    };
    raw.clamp(-2000.0, 2000.0)
}

/// Expand CSS `padding` shorthand (1–4 tokens) into horizontal `(left, right)` [`TextIndentSpec`].
/// Top/bottom are ignored for read layout. If any token is invalid, returns `(None, None)`.
/// Order: 1 → all sides; 2 → vertical, horizontal; 3 → top, horizontal, bottom; 4 → top, right, bottom, left.
fn parse_padding_shorthand_horizontal(value: &str) -> (Option<TextIndentSpec>, Option<TextIndentSpec>) {
    let tokens: Vec<&str> = trim_css_ascii_whitespace(value)
        .split_whitespace()
        .collect();
    if tokens.is_empty() || tokens.len() > 4 {
        return (None, None);
    }
    let p = |t: &str| parse_text_indent(t);
    let Some(t0) = p(tokens[0]) else {
        return (None, None);
    };
    match tokens.len() {
        1 => (Some(t0), Some(t0)),
        2 => {
            let Some(t1) = p(tokens[1]) else {
                return (None, None);
            };
            (Some(t1), Some(t1))
        }
        3 => {
            let Some(t1) = p(tokens[1]) else {
                return (None, None);
            };
            if p(tokens[2]).is_none() {
                return (None, None);
            }
            (Some(t1), Some(t1))
        }
        4 => {
            let Some(t1) = p(tokens[1]) else {
                return (None, None);
            };
            if p(tokens[2]).is_none() {
                return (None, None);
            }
            let Some(t3) = p(tokens[3]) else {
                return (None, None);
            };
            (Some(t3), Some(t1))
        }
        _ => (None, None),
    }
}

/// Resolve a non-negative horizontal inset (`padding-left` / `padding-right`). `%` uses `containing_width_px` (read-area width before inset).
pub fn resolve_padding_inset_px(
    spec: TextIndentSpec,
    used_font_size: f32,
    root_px: f32,
    containing_width_px: f32,
) -> f32 {
    let w = containing_width_px.max(1.0);
    let raw = match spec {
        TextIndentSpec::Px(v) => v,
        TextIndentSpec::Em(v) => v * used_font_size,
        TextIndentSpec::Rem(v) => v * root_px,
        TextIndentSpec::Percent(p) => p / 100.0 * w,
    };
    raw.clamp(0.0, 2000.0)
}

/// Author `max-width` lengths (not inherited in CSS — only matching rules set this on each node).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaxWidthSpec {
    /// `max-width: none` on this element (overrides a wider cascade value when present on the same element map).
    NoLimit,
    Px(f32),
    Em(f32),
    Rem(f32),
    Percent(f32),
}

fn clamp_max_width_px(px: f32) -> f32 {
    px.clamp(0.0, 8000.0)
}

pub fn parse_max_width(value: &str) -> Option<MaxWidthSpec> {
    let s = trim_css_ascii_whitespace(value).to_ascii_lowercase();
    if s.is_empty() {
        return None;
    }
    if s == "none" {
        return Some(MaxWidthSpec::NoLimit);
    }
    if let Some(rest) = s.strip_suffix("px") {
        let n: f32 = rest.trim().parse().ok()?;
        return n.is_finite().then_some(MaxWidthSpec::Px(n));
    }
    if s.ends_with("rem") {
        let n: f32 = s[..s.len() - 3].trim().parse().ok()?;
        return n.is_finite().then_some(MaxWidthSpec::Rem(n));
    }
    if s.ends_with("em") {
        let n: f32 = s[..s.len() - 2].trim().parse().ok()?;
        return n.is_finite().then_some(MaxWidthSpec::Em(n));
    }
    if s.ends_with('%') {
        let n: f32 = s[..s.len() - 1].trim().parse().ok()?;
        return n.is_finite().then_some(MaxWidthSpec::Percent(n));
    }
    None
}

/// Returns a cap width in px, or `None` when there is no limit (`NoLimit` or unresolved ≤ 0).
pub fn resolve_max_width_cap_px(
    spec: MaxWidthSpec,
    used_font_size: f32,
    root_px: f32,
    available_px: f32,
) -> Option<f32> {
    let avail = available_px.max(1.0);
    let px = match spec {
        MaxWidthSpec::NoLimit => return None,
        MaxWidthSpec::Px(v) => clamp_max_width_px(v),
        MaxWidthSpec::Em(e) => clamp_max_width_px(e * used_font_size),
        MaxWidthSpec::Rem(r) => clamp_max_width_px(r * root_px),
        MaxWidthSpec::Percent(p) => clamp_max_width_px(p / 100.0 * avail),
    };
    if px <= f32::EPSILON {
        None
    } else {
        Some(px.min(avail))
    }
}

fn titlecase_first_grapheme_cluster(g: &str) -> String {
    let mut it = g.chars();
    let Some(c0) = it.next() else {
        return String::new();
    };
    let mut s: String = c0.to_uppercase().collect();
    s.extend(it);
    s
}

/// CSS `text-transform: capitalize` — [CSS Text](https://www.w3.org/TR/css-text-3/#valdef-text-transform-capitalize):
/// first extended grapheme cluster of each UAX \#29 word that **starts** with an alphabetic
/// character is uppercased at its first scalar value; other grapheme clusters in that word
/// are copied verbatim.
fn capitalize_words_css(s: &str) -> String {
    use unicode_segmentation::UnicodeSegmentation;

    let mut out = String::with_capacity(s.len());
    for segment in s.split_word_bounds() {
        let gcs: Vec<&str> = segment.graphemes(true).collect();
        let word_starts_with_letter = gcs
            .first()
            .is_some_and(|g| g.chars().any(|c| c.is_alphabetic()));
        if word_starts_with_letter {
            for (i, g) in gcs.iter().enumerate() {
                if i == 0 {
                    out.push_str(&titlecase_first_grapheme_cluster(g));
                } else {
                    out.push_str(g);
                }
            }
        } else {
            out.push_str(segment);
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
    /// First-line inset; merged with `html`/`body`. Resolved to px at paint time via [`resolve_text_indent_px`].
    pub text_indent: Option<TextIndentSpec>,
    /// Opacity `0`…`1` (or `%` in author CSS); merged with `html`/`body`. Applied to the resolved text color when painting (not a full stacking-context / subtree model).
    pub opacity: Option<f32>,
    /// `visible` / `hidden` / `collapse` (see [`VisibilityHint`]); merged with `html`/`body`.
    pub visibility: Option<VisibilityHint>,
    /// `display: none` removes the node from read layout (no margins). Set by rules matching this node, or for **every** node when `html` / `body` **type** defaults ([`cascade_document_defaults`]) resolve to `display: none` (subtree hidden). Other `display` values on `html`/`body` are not copied onto children (non-inheritance).
    pub display: Option<DisplayHint>,
    /// `white-space` (`nowrap` vs `normal` / `pre-wrap` / `pre-line`); merged with `html`/`body`.
    pub white_space: Option<WhiteSpaceHint>,
    /// `word-break` (`break-all` vs `normal`); merged with `html`/`body`.
    pub word_break: Option<WordBreakHint>,
    /// `overflow-wrap` or legacy `word-wrap`; merged with `html`/`body`. If both names are set on the same element, `overflow-wrap` wins.
    pub overflow_wrap: Option<OverflowWrapHint>,
    /// `max-width` (`none`, `px` / `em` / `rem` / `%`). **Not** merged from `html`/`body` (non-inherited); only rules matching this node.
    pub max_width: Option<MaxWidthSpec>,
    /// `padding-left` (`px` / `em` / `rem` / `%`); **not** inherited. Longhand wins over the horizontal slice of `padding` shorthand when both are set.
    pub padding_left: Option<TextIndentSpec>,
    /// `padding-right`; same as [`Self::padding_left`].
    pub padding_right: Option<TextIndentSpec>,
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

/// Parse CSS `opacity`: unitless number `0`…`1` or percentage (`50` → `0.5`).
pub fn parse_opacity(value: &str) -> Option<f32> {
    let s = trim_css_ascii_whitespace(value);
    if s.is_empty() {
        return None;
    }
    let lower = s.to_ascii_lowercase();
    if let Some(rest) = lower.strip_suffix('%') {
        let n: f32 = rest.trim().parse().ok()?;
        return (n.is_finite()).then_some((n / 100.0).clamp(0.0, 1.0));
    }
    let n: f32 = lower.parse().ok()?;
    (n.is_finite()).then_some(n.clamp(0.0, 1.0))
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
    let doc = cascade_document_defaults(bundle);
    let doc_suppresses_layout = doc
        .get("display")
        .and_then(|v| parse_display(v))
        .is_some_and(|d| matches!(d, DisplayHint::None));
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
                .and_then(|v| parse_font_size_px(v, AUTHOR_STYLE_ROOT_PX))
                .map(clamp_font);
            let used_font_size = font_size.unwrap_or_else(|| default_font_size_px(n.kind));
            let line_height_px = merged_author_value(&m, &doc, "line-height")
                .and_then(|v| parse_line_height(v, used_font_size, AUTHOR_STYLE_ROOT_PX));
            let letter_spacing_px = merged_author_value(&m, &doc, "letter-spacing")
                .and_then(|v| parse_letter_spacing(v, used_font_size, AUTHOR_STYLE_ROOT_PX));
            let font_weight =
                merged_author_value(&m, &doc, "font-weight").and_then(parse_font_weight);
            let font_style_italic =
                merged_author_value(&m, &doc, "font-style").and_then(parse_font_style);
            // Margins are not inherited; longhands win over `margin` shorthand when present.
            let margin_top = resolve_margin_top(&m, AUTHOR_STYLE_ROOT_PX);
            let margin_bottom = resolve_margin_bottom(&m, AUTHOR_STYLE_ROOT_PX);
            let underline = merged_author_value(&m, &doc, "text-decoration")
                .and_then(parse_text_decoration_underline);
            let text_align = merged_author_value(&m, &doc, "text-align").and_then(parse_text_align);
            let text_transform =
                merged_author_value(&m, &doc, "text-transform").and_then(parse_text_transform);
            let text_indent = merged_author_value(&m, &doc, "text-indent").and_then(parse_text_indent);
            let opacity = merged_author_value(&m, &doc, "opacity").and_then(parse_opacity);
            let visibility = merged_author_value(&m, &doc, "visibility").and_then(parse_visibility);
            // `display` is not inherited onto arbitrary properties, but `html`/`body { display: none }`
            // suppresses the whole document subtree; approximate that by hiding every flattened node.
            let mut display = m.get("display").and_then(|v| parse_display(v));
            if doc_suppresses_layout {
                display = Some(DisplayHint::None);
            }
            let white_space =
                merged_author_value(&m, &doc, "white-space").and_then(parse_white_space);
            let word_break =
                merged_author_value(&m, &doc, "word-break").and_then(parse_word_break);
            let overflow_wrap = merged_overflow_wrap(&m, &doc).and_then(parse_overflow_wrap);
            let max_width = m.get("max-width").and_then(|v| parse_max_width(v));
            let (pad_sh_left, pad_sh_right) = m
                .get("padding")
                .map(String::as_str)
                .map(parse_padding_shorthand_horizontal)
                .unwrap_or((None, None));
            let padding_left = m
                .get("padding-left")
                .map(String::as_str)
                .and_then(parse_text_indent)
                .or(pad_sh_left);
            let padding_right = m
                .get("padding-right")
                .map(String::as_str)
                .and_then(parse_text_indent)
                .or(pad_sh_right);
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
                text_indent,
                opacity,
                visibility,
                display,
                white_space,
                word_break,
                overflow_wrap,
                max_width,
                padding_left,
                padding_right,
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
            "Hello WORLD"
        );
        assert_eq!(
            display_text_cow("the (quick) brown", Some(TextTransformHint::Capitalize)).as_ref(),
            "The (Quick) Brown"
        );
        assert_eq!(
            display_text_cow("don't stop", Some(TextTransformHint::Capitalize)).as_ref(),
            "Don't Stop"
        );
        assert_eq!(
            display_text_cow("32nd place", Some(TextTransformHint::Capitalize)).as_ref(),
            "32nd Place"
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
    fn display_text_capitalize_respects_grapheme_clusters() {
        // e + combining acute is one extended grapheme cluster; only base is uppercased.
        let s = "e\u{301}lan";
        let out = display_text_cow(s, Some(TextTransformHint::Capitalize));
        assert!(out.starts_with('E'));
        assert!(out.contains("\u{301}"));
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

    #[test]
    fn parse_text_indent_units() {
        assert_eq!(parse_text_indent("2em"), Some(TextIndentSpec::Em(2.0)));
        assert_eq!(parse_text_indent("-12px"), Some(TextIndentSpec::Px(-12.0)));
        assert_eq!(parse_text_indent("10%"), Some(TextIndentSpec::Percent(10.0)));
        assert_eq!(parse_text_indent("1.5rem"), Some(TextIndentSpec::Rem(1.5)));
    }

    #[test]
    fn resolve_text_indent_percent_uses_line_width() {
        let px = resolve_text_indent_px(TextIndentSpec::Percent(10.0), 15.0, 16.0, 400.0);
        assert!((px - 40.0).abs() < 0.01);
    }

    #[test]
    fn text_indent_inherited_from_body() {
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
                    property: "text-indent".into(),
                    value_display: "20px".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].text_indent, Some(TextIndentSpec::Px(20.0)));
    }

    #[test]
    fn parse_opacity_number_and_percent() {
        assert_eq!(parse_opacity("0.5"), Some(0.5));
        assert_eq!(parse_opacity("50%"), Some(0.5));
        assert_eq!(parse_opacity("1"), Some(1.0));
        assert_eq!(parse_opacity("150%"), Some(1.0));
        assert_eq!(parse_opacity("-1"), Some(0.0));
        assert_eq!(parse_opacity("inherit"), None);
    }

    #[test]
    fn opacity_inherited_from_body() {
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
                    property: "opacity".into(),
                    value_display: "0.25".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].opacity, Some(0.25));
    }

    #[test]
    fn parse_visibility_keywords() {
        assert_eq!(parse_visibility("visible"), Some(VisibilityHint::Visible));
        assert_eq!(parse_visibility("HIDDEN"), Some(VisibilityHint::Hidden));
        assert_eq!(parse_visibility("collapse"), Some(VisibilityHint::Hidden));
        assert_eq!(parse_visibility("bogus"), None);
    }

    #[test]
    fn visibility_hidden_inherited_from_body() {
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
                    property: "visibility".into(),
                    value_display: "hidden".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].visibility, Some(VisibilityHint::Hidden));
    }

    #[test]
    fn visibility_visible_on_element_overrides_body() {
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
                        property: "visibility".into(),
                        value_display: "hidden".into(),
                    }],
                },
                ParsedQualifiedRule {
                    prelude_display: "p".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "visibility".into(),
                        value_display: "visible".into(),
                    }],
                },
            ],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].visibility, Some(VisibilityHint::Visible));
    }

    #[test]
    fn parse_display_none_and_flow() {
        assert_eq!(parse_display("none"), Some(DisplayHint::None));
        assert_eq!(parse_display("BLOCK"), Some(DisplayHint::Flow));
        assert_eq!(parse_display("flex"), Some(DisplayHint::Flow));
        assert_eq!(parse_display("contents"), Some(DisplayHint::Flow));
        assert_eq!(parse_display("bogus"), None);
    }

    #[test]
    fn display_none_from_matching_rule() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "Hi".into(),
            href: None,
            classes: vec!["x".into()],
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![ParsedQualifiedRule {
                prelude_display: ".x".into(),
                declarations: vec![SimpleDeclaration {
                    property: "display".into(),
                    value_display: "none".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].display, Some(DisplayHint::None));
    }

    #[test]
    fn body_type_display_none_hides_all_nodes() {
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
                    property: "display".into(),
                    value_display: "none".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].display, Some(DisplayHint::None));
    }

    #[test]
    fn html_type_display_none_hides_all_nodes() {
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
                prelude_display: "html".into(),
                declarations: vec![SimpleDeclaration {
                    property: "display".into(),
                    value_display: "none".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].display, Some(DisplayHint::None));
    }

    #[test]
    fn later_html_display_block_overrides_body_display_none_in_defaults() {
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
                        property: "display".into(),
                        value_display: "none".into(),
                    }],
                },
                ParsedQualifiedRule {
                    prelude_display: "html".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "display".into(),
                        value_display: "block".into(),
                    }],
                },
            ],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].display, None);
    }

    #[test]
    fn body_type_display_block_does_not_set_display_on_paragraph() {
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
                    property: "display".into(),
                    value_display: "block".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].display, None);
    }

    #[test]
    fn parse_white_space_keywords() {
        assert_eq!(parse_white_space("nowrap"), Some(WhiteSpaceHint::Nowrap));
        assert_eq!(parse_white_space("NORMAL"), Some(WhiteSpaceHint::Normal));
        assert_eq!(parse_white_space("pre-wrap"), Some(WhiteSpaceHint::Normal));
        assert_eq!(parse_white_space("pre"), None);
        assert_eq!(parse_white_space("break-spaces"), None);
    }

    #[test]
    fn white_space_nowrap_merged_from_body() {
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
                    property: "white-space".into(),
                    value_display: "nowrap".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].white_space, Some(WhiteSpaceHint::Nowrap));
    }

    #[test]
    fn white_space_normal_on_p_overrides_body_nowrap() {
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
                        property: "white-space".into(),
                        value_display: "nowrap".into(),
                    }],
                },
                ParsedQualifiedRule {
                    prelude_display: "p".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "white-space".into(),
                        value_display: "normal".into(),
                    }],
                },
            ],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].white_space, Some(WhiteSpaceHint::Normal));
    }

    #[test]
    fn parse_word_break_keywords() {
        assert_eq!(parse_word_break("break-all"), Some(WordBreakHint::BreakAll));
        assert_eq!(parse_word_break("normal"), Some(WordBreakHint::Normal));
        assert_eq!(parse_word_break("keep-all"), None);
    }

    #[test]
    fn parse_overflow_wrap_keywords() {
        assert_eq!(
            parse_overflow_wrap("anywhere"),
            Some(OverflowWrapHint::Anywhere)
        );
        assert_eq!(
            parse_overflow_wrap("break-word"),
            Some(OverflowWrapHint::BreakWord)
        );
        assert_eq!(parse_overflow_wrap("normal"), Some(OverflowWrapHint::Normal));
        assert_eq!(parse_overflow_wrap("bogus"), None);
    }

    #[test]
    fn word_break_break_all_from_body() {
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
                    property: "word-break".into(),
                    value_display: "break-all".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].word_break, Some(WordBreakHint::BreakAll));
    }

    #[test]
    fn legacy_word_wrap_anywhere_maps_to_overflow_wrap_hint() {
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
                    property: "word-wrap".into(),
                    value_display: "anywhere".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].overflow_wrap, Some(OverflowWrapHint::Anywhere));
    }

    #[test]
    fn parse_max_width_none_and_units() {
        assert_eq!(parse_max_width("none"), Some(MaxWidthSpec::NoLimit));
        assert_eq!(parse_max_width("200px"), Some(MaxWidthSpec::Px(200.0)));
        assert_eq!(parse_max_width("10em"), Some(MaxWidthSpec::Em(10.0)));
        assert_eq!(parse_max_width("50%"), Some(MaxWidthSpec::Percent(50.0)));
        assert_eq!(parse_max_width("min-content"), None);
    }

    #[test]
    fn resolve_max_width_percent_uses_available_width() {
        let cap = resolve_max_width_cap_px(
            MaxWidthSpec::Percent(40.0),
            16.0,
            AUTHOR_STYLE_ROOT_PX,
            500.0,
        );
        assert!((cap.unwrap() - 200.0).abs() < 0.01);
    }

    #[test]
    fn max_width_from_matching_rule_not_from_body() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "Hi".into(),
            href: None,
            classes: vec!["narrow".into()],
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![
                ParsedQualifiedRule {
                    prelude_display: "body".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "max-width".into(),
                        value_display: "100px".into(),
                    }],
                },
                ParsedQualifiedRule {
                    prelude_display: ".narrow".into(),
                    declarations: vec![SimpleDeclaration {
                        property: "max-width".into(),
                        value_display: "240px".into(),
                    }],
                },
            ],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].max_width, Some(MaxWidthSpec::Px(240.0)));
    }

    #[test]
    fn body_max_width_does_not_fill_paragraph_hint() {
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
                    property: "max-width".into(),
                    value_display: "100px".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].max_width, None);
    }

    #[test]
    fn resolve_padding_inset_percent() {
        let px = resolve_padding_inset_px(
            TextIndentSpec::Percent(10.0),
            16.0,
            AUTHOR_STYLE_ROOT_PX,
            800.0,
        );
        assert!((px - 80.0).abs() < 0.01);
    }

    #[test]
    fn padding_left_from_class_rule() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "Hi".into(),
            href: None,
            classes: vec!["pad".into()],
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![ParsedQualifiedRule {
                prelude_display: ".pad".into(),
                declarations: vec![
                    SimpleDeclaration {
                        property: "padding-left".into(),
                        value_display: "24px".into(),
                    },
                    SimpleDeclaration {
                        property: "padding-right".into(),
                        value_display: "8px".into(),
                    },
                ],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].padding_left, Some(TextIndentSpec::Px(24.0)));
        assert_eq!(hints[0].padding_right, Some(TextIndentSpec::Px(8.0)));
    }

    #[test]
    fn body_padding_longhands_do_not_fill_paragraph_hint() {
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
                    property: "padding-left".into(),
                    value_display: "40px".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].padding_left, None);
    }

    #[test]
    fn padding_shorthand_one_value_sets_left_and_right() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "Hi".into(),
            href: None,
            classes: vec!["x".into()],
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![ParsedQualifiedRule {
                prelude_display: ".x".into(),
                declarations: vec![SimpleDeclaration {
                    property: "padding".into(),
                    value_display: "12px".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].padding_left, Some(TextIndentSpec::Px(12.0)));
        assert_eq!(hints[0].padding_right, Some(TextIndentSpec::Px(12.0)));
    }

    #[test]
    fn padding_shorthand_four_values_top_right_bottom_left() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "Hi".into(),
            href: None,
            classes: vec!["x".into()],
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![ParsedQualifiedRule {
                prelude_display: ".x".into(),
                declarations: vec![SimpleDeclaration {
                    property: "padding".into(),
                    value_display: "1px 2px 3px 4px".into(),
                }],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].padding_left, Some(TextIndentSpec::Px(4.0)));
        assert_eq!(hints[0].padding_right, Some(TextIndentSpec::Px(2.0)));
    }

    #[test]
    fn padding_left_longhand_overrides_shorthand() {
        let nodes = vec![DomNode {
            kind: DomNodeType::Paragraph,
            text: "Hi".into(),
            href: None,
            classes: vec!["x".into()],
            element_id: None,
        }];
        let bundle = vec![(
            "https://example.com/a.css".into(),
            vec![ParsedQualifiedRule {
                prelude_display: ".x".into(),
                declarations: vec![
                    SimpleDeclaration {
                        property: "padding".into(),
                        value_display: "1px 2px 3px 4px".into(),
                    },
                    SimpleDeclaration {
                        property: "padding-left".into(),
                        value_display: "40px".into(),
                    },
                ],
            }],
        )];
        let hints = compute_dom_paint_hints(&nodes, &bundle);
        assert_eq!(hints[0].padding_left, Some(TextIndentSpec::Px(40.0)));
        assert_eq!(hints[0].padding_right, Some(TextIndentSpec::Px(2.0)));
    }
}
