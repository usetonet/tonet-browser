//! Renders the simplified DOM into egui widgets.

use crate::css_resolve::DomNodePaintHints;
use crate::i18n;
use crate::i18n::Locale;
use crate::parser::{DomNode, DomNodeType};
use crate::theme;
use egui::{RichText, Ui};

/// Draws parsed nodes in the scrollable page area. `link_target` receives an absolute URL when a link is activated.
///
/// When `author_hints` is `Some` and has the same length as `nodes`, `color` / `font-size` from author
/// stylesheets (simple type selectors) override the built-in theme defaults per node.
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
                ui.add_space(4.0);
                ui.label(RichText::new(&node.text).strong().size(size).color(color));
                ui.add_space(8.0);
            }
            DomNodeType::H1 => {
                let (def_size, def_color) = (26.0, theme::body_text());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                ui.add_space(6.0);
                ui.label(RichText::new(&node.text).strong().size(size).color(color));
                ui.add_space(4.0);
            }
            DomNodeType::H2 => {
                let (def_size, def_color) = (19.0, theme::body_text());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                ui.add_space(4.0);
                ui.label(RichText::new(&node.text).strong().size(size).color(color));
                ui.add_space(2.0);
            }
            DomNodeType::Paragraph => {
                let (def_size, def_color) = (15.0, theme::body_text());
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                ui.label(RichText::new(&node.text).size(size).color(color));
                ui.add_space(6.0);
            }
            DomNodeType::Link => {
                let (def_size, def_color) = if node.href.is_some() {
                    (15.0, theme::link())
                } else {
                    (15.0, theme::body_text())
                };
                let size = hint.and_then(|h| h.font_size).unwrap_or(def_size);
                let color = hint.and_then(|h| h.color).unwrap_or(def_color);
                if let Some(ref href) = node.href {
                    let r = ui.link(RichText::new(&node.text).size(size).color(color));
                    if r.clicked() {
                        *link_target = Some(href.clone());
                    }
                    r.on_hover_text(href);
                } else {
                    ui.label(RichText::new(&node.text).size(size).color(color));
                }
                ui.add_space(4.0);
            }
        }
    }
}
