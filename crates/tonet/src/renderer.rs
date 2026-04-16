//! Renders the simplified DOM into egui widgets.

use crate::css_resolve::{DomNodePaintHints, TextAlignHint};
use crate::i18n;
use crate::i18n::Locale;
use crate::parser::{DomNode, DomNodeType};
use crate::theme;
use egui::{Align, Color32, Layout, RichText, Ui};

/// Draws parsed nodes in the scrollable page area. `link_target` receives an absolute URL when a link is activated.
///
/// When `author_hints` is `Some` and has the same length as `nodes`, author `color`, `font-size`,
/// `line-height`, `letter-spacing`, `font-weight`, `font-style`, `margin` / margins, `text-decoration`, and `text-align` override or extend built-in page chrome.
pub fn render_nodes(
    ui: &mut Ui,
    loc: Locale,
    nodes: &[DomNode],
    author_hints: Option<&[DomNodePaintHints]>,
    link_target: &mut Option<String>,
) {
    if nodes.is_empty() {
        ui.label(
            RichText::new(i18n::empty_page_hint(loc))
                .italics()
                .color(theme::loading_muted()),
        );
        return;
    }

    let hints_slice = author_hints.filter(|h| h.len() == nodes.len());

    for (i, node) in nodes.iter().enumerate() {
        let hint = hints_slice.and_then(|h| h.get(i)).copied();

        match node.kind {
            DomNodeType::Title => {
                let (def_size, def_color) = (21.0, theme::page_title());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                let rt = styled_rich_text(node, hint, size, color);
                with_text_align(ui, hint, |ui| {
                    ui.label(rt);
                });
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
            DomNodeType::H1 => {
                let (def_size, def_color) = (26.0, theme::body_text());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                let rt = styled_rich_text(node, hint, size, color);
                with_text_align(ui, hint, |ui| {
                    ui.label(rt);
                });
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
            DomNodeType::H2 => {
                let (def_size, def_color) = (19.0, theme::body_text());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                let rt = styled_rich_text(node, hint, size, color);
                with_text_align(ui, hint, |ui| {
                    ui.label(rt);
                });
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
            DomNodeType::Paragraph => {
                let (def_size, def_color) = (15.0, theme::body_text());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                let rt = styled_rich_text(node, hint, size, color);
                with_text_align(ui, hint, |ui| {
                    ui.label(rt);
                });
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
            DomNodeType::Link => {
                let (def_size, def_color) = if node.href.is_some() {
                    (15.0, theme::link())
                } else {
                    (15.0, theme::body_text())
                };
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                let top = hint
                    .and_then(|h| h.margin_top)
                    .unwrap_or_else(|| default_margin_top(node.kind));
                ui.add_space(top);
                let rt = styled_rich_text(node, hint, size, color);
                with_text_align(ui, hint, |ui| {
                    if let Some(ref href) = node.href {
                        let r = ui.link(rt.clone());
                        if r.clicked() {
                            *link_target = Some(href.clone());
                        }
                        r.on_hover_text(href);
                    } else {
                        ui.label(rt);
                    }
                });
                let bottom = hint
                    .and_then(|h| h.margin_bottom)
                    .unwrap_or_else(|| default_margin_bottom(node.kind));
                ui.add_space(bottom);
            }
        }
    }
}

fn with_text_align(ui: &mut Ui, hint: Option<DomNodePaintHints>, child: impl FnOnce(&mut Ui)) {
    let align = hint
        .and_then(|h| h.text_align)
        .unwrap_or(TextAlignHint::Start);
    let w = ui.available_width();
    match align {
        TextAlignHint::Start => child(ui),
        TextAlignHint::Center => {
            ui.allocate_ui_with_layout(
                egui::vec2(w, 0.0),
                Layout::top_down(Align::Center),
                |ui| {
                    ui.set_width(w);
                    child(ui);
                },
            );
        }
        TextAlignHint::End => {
            ui.allocate_ui_with_layout(
                egui::vec2(w, 0.0),
                Layout::right_to_left(Align::Min),
                |ui| {
                    ui.set_width(w);
                    child(ui);
                },
            );
        }
    }
}

fn default_margin_top(kind: DomNodeType) -> f32 {
    match kind {
        DomNodeType::Title => 4.0,
        DomNodeType::H1 => 6.0,
        DomNodeType::H2 => 4.0,
        DomNodeType::Paragraph | DomNodeType::Link => 0.0,
    }
}

fn default_margin_bottom(kind: DomNodeType) -> f32 {
    match kind {
        DomNodeType::Title => 8.0,
        DomNodeType::H1 => 4.0,
        DomNodeType::H2 => 2.0,
        DomNodeType::Paragraph => 6.0,
        DomNodeType::Link => 4.0,
    }
}

/// Title / headings default to bold (`strong`); body text does not. Author `font-weight` / `font-style` override.
fn styled_rich_text(
    node: &DomNode,
    hint: Option<DomNodePaintHints>,
    size: f32,
    color: Color32,
) -> RichText {
    let default_bold = matches!(
        node.kind,
        DomNodeType::Title | DomNodeType::H1 | DomNodeType::H2
    );
    let default_weight = if default_bold { 700u16 } else { 400u16 };
    let weight = hint.and_then(|h| h.font_weight).unwrap_or(default_weight);

    let mut rt = RichText::new(&node.text).size(size).color(color);
    if weight >= 600 {
        rt = rt.strong();
    }
    if matches!(hint.and_then(|h| h.font_style_italic), Some(true)) {
        rt = rt.italics();
    }
    if matches!(hint.and_then(|h| h.underline), Some(true)) {
        rt = rt.underline();
    }
    if let Some(lh) = hint.and_then(|h| h.line_height_px) {
        rt = rt.line_height(Some(lh));
    }
    if let Some(ls) = hint.and_then(|h| h.letter_spacing_px) {
        rt = rt.extra_letter_spacing(ls);
    }
    rt
}
