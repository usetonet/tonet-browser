//! Elements panel — DOM tree from Servo `evaluate_javascript` snapshot.

use egui::{self, RichText, Ui};

use crate::i18n::{self, Locale};
use crate::servo_engine::embedder_devtools::ServoDomTreeNode;
use crate::theme;

fn node_label(n: &ServoDomTreeNode) -> String {
    if n.tag == "text" {
        return format!(
            "#text \"{}\"",
            n.text_preview.as_deref().unwrap_or("")
        );
    }
    let mut s = format!("<{}", n.tag);
    if !n.id_attr.is_empty() {
        s.push('#');
        s.push_str(&n.id_attr);
    }
    if !n.class_attr.is_empty() {
        let cls = n.class_attr.split_whitespace().next().unwrap_or("");
        if !cls.is_empty() {
            s.push('.');
            s.push_str(cls);
        }
        if n.class_attr.split_whitespace().count() > 1 {
            s.push_str("…");
        }
    }
    s.push('>');
    s
}

fn show_dom_node(ui: &mut Ui, path: &str, n: &ServoDomTreeNode, depth: usize) {
    let label = node_label(n);
    if n.children.is_empty() {
        ui.label(
            RichText::new(format!("{label}"))
                .family(egui::FontFamily::Monospace)
                .size(12.0)
                .color(theme::devtools_tab_text_idle()),
        );
        return;
    }
    egui::CollapsingHeader::new(
        RichText::new(label)
            .family(egui::FontFamily::Monospace)
            .size(12.0)
            .color(theme::devtools_tab_text_active()),
    )
    .default_open(depth < 3)
    .show(ui, |ui| {
        for (i, child) in n.children.iter().enumerate() {
            let child_path = format!("{path}>{i}");
            show_dom_node(ui, &child_path, child, depth + 1);
        }
    });
}

pub fn show_elements_panel(
    ui: &mut Ui,
    loc: Locale,
    root: Option<&ServoDomTreeNode>,
    dom_error: Option<&str>,
    dom_loading: bool,
    on_refresh_dom: &mut dyn FnMut(),
) {
    ui.horizontal(|ui| {
        if ui
            .small_button(i18n::devtools_elements_refresh(loc))
            .clicked()
        {
            on_refresh_dom();
        }
        if dom_loading {
            ui.label(
                RichText::new(i18n::devtools_elements_loading(loc))
                    .small()
                    .color(theme::loading_muted()),
            );
        }
    });
    ui.add_space(theme::SP);

    if let Some(err) = dom_error {
        ui.label(
            RichText::new(err)
                .small()
                .color(theme::error_title()),
        );
        return;
    }

    if let Some(root) = root {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.style_mut().override_font_id =
                    Some(egui::FontId::monospace(12.0));
                show_dom_node(ui, "0", root, 0);
            });
    } else if !dom_loading {
        ui.label(
            RichText::new(i18n::devtools_elements_empty(loc))
                .small()
                .color(theme::loading_muted()),
        );
        let auto_id = egui::Id::new("tonet_dt_dom_auto_refresh");
        let did_auto = ui.ctx().data(|d| d.get_temp::<bool>(auto_id).unwrap_or(false));
        if !did_auto {
            ui.ctx().data_mut(|d| d.insert_temp(auto_id, true));
            on_refresh_dom();
        }
    }
}
