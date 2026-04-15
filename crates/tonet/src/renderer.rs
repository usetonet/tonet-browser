//! Renders the simplified DOM into egui widgets.

use crate::i18n::Locale;
use crate::i18n;
use crate::parser::{DomNode, DomNodeType};
use crate::theme;
use egui::{RichText, Ui};

/// Draws parsed nodes in the scrollable page area. `link_target` receives an absolute URL when a link is activated.
pub fn render_nodes(ui: &mut Ui, loc: Locale, nodes: &[DomNode], link_target: &mut Option<String>) {
    if nodes.is_empty() {
        ui.label(
            RichText::new(i18n::empty_page_hint(loc))
                .italics()
                .color(theme::loading_muted()),
        );
        return;
    }

    for node in nodes {
        match node.kind {
            DomNodeType::Title => {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(&node.text)
                        .strong()
                        .size(21.0)
                        .color(theme::page_title()),
                );
                ui.add_space(8.0);
            }
            DomNodeType::H1 => {
                ui.add_space(6.0);
                ui.label(RichText::new(&node.text).strong().size(26.0).color(theme::body_text()));
                ui.add_space(4.0);
            }
            DomNodeType::H2 => {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(&node.text)
                        .strong()
                        .size(19.0)
                        .color(theme::body_text()),
                );
                ui.add_space(2.0);
            }
            DomNodeType::Paragraph => {
                ui.label(RichText::new(&node.text).size(15.0).color(theme::body_text()));
                ui.add_space(6.0);
            }
            DomNodeType::Link => {
                if let Some(ref href) = node.href {
                    let r = ui.link(RichText::new(&node.text).size(15.0).color(theme::link()));
                    if r.clicked() {
                        *link_target = Some(href.clone());
                    }
                    r.on_hover_text(href);
                } else {
                    ui.label(RichText::new(&node.text).size(15.0).color(theme::body_text()));
                }
                ui.add_space(4.0);
            }
        }
    }
}
