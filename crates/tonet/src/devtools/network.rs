//! Network panel — requests seen in Servo `load_web_resource`.

use egui::{self, RichText, Ui};

use crate::i18n::{self, Locale};
use crate::servo_engine::embedder_devtools::{NetworkEntryStatus, ServoNetworkEntry};
use crate::theme;

pub fn show_network_panel(
    ui: &mut Ui,
    loc: Locale,
    entries: &[ServoNetworkEntry],
    on_clear_network: &mut dyn FnMut(),
) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(i18n::devtools_network_hint(loc))
                .small()
                .color(theme::loading_muted()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui
                .small_button(i18n::servo_page_console_clear(loc))
                .clicked()
            {
                on_clear_network();
            }
        });
    });
    ui.add_space(theme::SP2);

    egui::Grid::new(egui::Id::new("tonet_dt_network_grid"))
        .num_columns(4)
        .spacing([theme::SP2, theme::SP])
        .striped(true)
        .show(ui, |ui| {
            ui.label(RichText::new(i18n::devtools_network_col_name(loc)).strong().small());
            ui.label(RichText::new(i18n::devtools_network_col_method(loc)).strong().small());
            ui.label(RichText::new(i18n::devtools_network_col_type(loc)).strong().small());
            ui.label(RichText::new(i18n::devtools_network_col_status(loc)).strong().small());
            ui.end_row();

            if entries.is_empty() {
                ui.label(
                    RichText::new(i18n::devtools_network_empty(loc))
                        .small()
                        .color(theme::loading_muted()),
                );
                ui.end_row();
                return;
            }

            for e in entries.iter().rev() {
                let name = short_url_display(&e.url);
                ui.label(
                    RichText::new(name)
                        .small()
                        .family(egui::FontFamily::Monospace)
                        .color(theme::accent()),
                )
                .on_hover_text(&e.url);
                ui.label(RichText::new(&e.method).small().monospace());
                ui.label(RichText::new(e.kind.label()).small());
                let status_color = match e.status {
                    NetworkEntryStatus::Pending => theme::loading_muted(),
                    NetworkEntryStatus::Sent => theme::accent(),
                    NetworkEntryStatus::Intercepted => color32_warn(),
                };
                ui.label(
                    RichText::new(e.status.label())
                        .small()
                        .color(status_color),
                );
                ui.end_row();
            }
        });
}

fn color32_warn() -> egui::Color32 {
    egui::Color32::from_rgb(230, 178, 92)
}

fn short_url_display(url: &str) -> String {
    if let Some(i) = url.find("://") {
        let rest = &url[i + 3..];
        if rest.len() > 72 {
            return format!("{}…", &rest[..72]);
        }
        return rest.to_string();
    }
    if url.len() > 80 {
        format!("{}…", &url[..80])
    } else {
        url.to_string()
    }
}
