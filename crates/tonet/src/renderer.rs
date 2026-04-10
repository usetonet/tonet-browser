//! Renders the simplified DOM into egui widgets.

use crate::i18n::Locale;
use crate::i18n;
use crate::parser::{DomNode, DomNodeType};
use egui::{Color32, RichText, Ui};

/// Draws parsed nodes in the scrollable page area.
pub fn render_nodes(ui: &mut Ui, loc: Locale, nodes: &[DomNode]) {
    if nodes.is_empty() {
        ui.label(
            RichText::new(i18n::empty_page_hint(loc))
                .italics()
                .color(Color32::GRAY),
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
                        .size(22.0)
                        .color(Color32::from_rgb(40, 90, 160)),
                );
                ui.add_space(8.0);
            }
            DomNodeType::H1 => {
                ui.add_space(6.0);
                ui.label(RichText::new(&node.text).strong().size(28.0));
                ui.add_space(4.0);
            }
            DomNodeType::H2 => {
                ui.add_space(4.0);
                ui.label(RichText::new(&node.text).strong().size(20.0));
                ui.add_space(2.0);
            }
            DomNodeType::Paragraph => {
                ui.label(RichText::new(&node.text).size(15.0));
                ui.add_space(6.0);
            }
        }
    }
}
