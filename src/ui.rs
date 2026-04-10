//! Componentes de interfaz reutilizables (barra superior y paneles de estado).
//!
//! Mantiene la vista separada de la lógica de `app.rs` para facilitar extensiones.

use egui::{Align, Color32, Layout, RichText, Ui};

/// Barra superior con campo de URL y botón de navegación.
/// Devuelve `true` si el usuario solicitó ir a la URL (botón o Enter).
pub fn show_top_bar(ui: &mut Ui, url_input: &mut String, loading: bool) -> bool {
    let mut go = false;
    ui.horizontal(|ui| {
        ui.label(RichText::new("URL").small());
        let response = ui.add_sized(
            [ui.available_width() - 96.0, 24.0],
            egui::TextEdit::singleline(url_input)
                .hint_text("https://ejemplo.com")
                .desired_rows(1),
        );

        // Enter con foco en el campo dispara la navegación (como un navegador clásico).
        if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            go = true;
        }

        let btn = ui.add_enabled(!loading, egui::Button::new(if loading { "…" } else { "Ir" }));
        if btn.clicked() {
            go = true;
        }
    });
    go
}

/// Muestra un mensaje de error con estilo visible (fondo suave y texto destacado).
pub fn show_error_panel(ui: &mut Ui, message: &str) {
    egui::Frame::default()
        .fill(Color32::from_rgb(60, 20, 20))
        .inner_margin(12.0)
        .rounding(6.0)
        .show(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.label(RichText::new("Error").strong().color(Color32::LIGHT_RED));
                ui.add_space(4.0);
                ui.label(RichText::new(message).color(Color32::from_rgb(255, 200, 200)));
            });
        });
}

/// Indicador de carga centrado.
pub fn show_loading(ui: &mut Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(24.0);
        ui.spinner();
        ui.label(RichText::new("Cargando…").size(18.0).strong());
        ui.add_space(8.0);
    });
}
