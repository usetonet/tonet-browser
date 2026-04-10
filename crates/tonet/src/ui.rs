//! Componentes de interfaz reutilizables (barra superior, estados, ajustes, avisos).

use egui::{Align, Color32, Layout, RichText, Ui, Vec2};

use crate::settings::{AppSettings, UpdatePolicy};

/// Resultado de la barra superior: navegar y/o abrir ajustes.
pub struct TopBarResult {
    pub navigate: bool,
    pub open_settings: bool,
}

/// Barra superior: URL, Ir, ajustes.
pub fn show_top_bar(ui: &mut Ui, url_input: &mut String, loading: bool) -> TopBarResult {
    let mut navigate = false;
    let mut open_settings = false;

    ui.horizontal(|ui| {
        ui.label(RichText::new("URL").small().color(Color32::from_gray(200)));
        let url_response = ui.add_sized(
            [ui.available_width() - 140.0, 26.0],
            egui::TextEdit::singleline(url_input)
                .hint_text("https://ejemplo.com")
                .desired_rows(1),
        );

        if url_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            navigate = true;
        }

        let go = ui.add_sized(
            Vec2::new(52.0, 28.0),
            egui::Button::new(RichText::new(if loading { "…" } else { "Ir" }).strong()),
        );
        if go.clicked() {
            navigate = true;
        }

        let settings_btn = ui
            .add_sized(Vec2::new(36.0, 28.0), egui::Button::new(RichText::new("⚙").size(16.0)))
            .on_hover_text("Ajustes (atajo: Ctrl o ⌘ + coma)");
        if settings_btn.clicked() {
            open_settings = true;
        }
    });

    TopBarResult {
        navigate,
        open_settings,
    }
}

pub fn show_error_panel(ui: &mut Ui, message: &str) {
    egui::Frame::default()
        .fill(Color32::from_rgb(72, 28, 28))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(140, 60, 60)))
        .inner_margin(14.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.label(RichText::new("Algo salió mal").strong().color(Color32::from_rgb(255, 160, 160)));
                ui.add_space(6.0);
                ui.label(RichText::new(message).color(Color32::from_rgb(255, 220, 220)));
            });
        });
}

pub fn show_loading(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(32.0);
        ui.spinner();
        ui.add_space(10.0);
        ui.label(RichText::new("Cargando…").size(18.0).strong());
        ui.add_space(6.0);
        ui.label(
            RichText::new("Tonet solo acepta páginas ligeras (≤ 1 MB).")
                .small()
                .color(Color32::GRAY),
        );
    });
}

pub fn show_update_banner(
    ui: &mut Ui,
    version_label: &str,
    on_open_downloads: impl FnOnce(),
    on_dismiss: impl FnOnce(),
) {
    egui::Frame::default()
        .fill(Color32::from_rgb(28, 52, 88))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 120, 200)))
        .inner_margin(12.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Nueva versión disponible").strong().color(Color32::WHITE));
                ui.label(
                    RichText::new(version_label)
                        .strong()
                        .color(Color32::from_rgb(180, 210, 255)),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button("Ocultar").clicked() {
                        on_dismiss();
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new("Descargar").strong())
                                .fill(Color32::from_rgb(70, 130, 220)),
                        )
                        .clicked()
                    {
                        on_open_downloads();
                    }
                });
            });
        });
}

#[allow(clippy::too_many_arguments)]
pub fn show_settings_window(
    ctx: &egui::Context,
    open: &mut bool,
    settings: &mut AppSettings,
    update_busy: bool,
    status_line: &str,
    current_version: &str,
    mut on_save: impl FnMut(&AppSettings),
    mut on_check_now: impl FnMut(),
) {
    let mut win = egui::Window::new("Ajustes")
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_width(440.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO);

    win = win.frame(
        egui::Frame::window(&ctx.style())
            .fill(Color32::from_rgb(36, 38, 42))
            .rounding(10.0),
    );

    win.show(ctx, |ui| {
        ui.add_space(4.0);
        ui.label(
            RichText::new("Actualizaciones")
                .size(18.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(format!("Versión instalada: {current_version}"))
                .small()
                .color(Color32::LIGHT_GRAY),
        );
        ui.add_space(12.0);

        ui.label(
            RichText::new("¿Cuándo debe comprobar Tonet si hay versiones nuevas en GitHub?")
                .color(Color32::from_gray(220)),
        );
        ui.add_space(10.0);

        for policy in [
            UpdatePolicy::OnStartup,
            UpdatePolicy::Periodic,
            UpdatePolicy::ManualOnly,
        ] {
            let label = AppSettings::update_policy_label(policy);
            let help = AppSettings::update_policy_help(policy);
            ui.radio_value(&mut settings.update_policy, policy, label)
                .on_hover_text(help);
            ui.label(RichText::new(help).small().color(Color32::GRAY).italics());
            ui.add_space(8.0);
        }

        ui.separator();
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let can_check = !update_busy;
            let r = ui.add_enabled(
                can_check,
                egui::Button::new(RichText::new("Comprobar ahora").strong()),
            );
            if r.clicked() {
                on_check_now();
            }
            if !can_check {
                r.on_disabled_hover_text("Ya hay una comprobación en curso…");
            }
            if update_busy {
                ui.spinner();
                ui.label(RichText::new("Consultando…").small().color(Color32::GRAY));
            }
        });

        ui.add_space(8.0);
        if !status_line.is_empty() {
            egui::Frame::default()
                .fill(Color32::from_rgb(30, 32, 36))
                .inner_margin(10.0)
                .rounding(6.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(status_line).color(Color32::from_gray(210)));
                });
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("Guardar preferencias").clicked() {
                on_save(settings);
            }
            if ui
                .button("Abrir página de descargas")
                .on_hover_text("Abre el navegador del sistema en GitHub Releases")
                .clicked()
            {
                crate::update::open_downloads_page();
            }
        });
    });
}
