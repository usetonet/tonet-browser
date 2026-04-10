//! Punto de entrada de Tonet — navegador minimalista (MVP).
//!
//! Arranca el runtime nativo de eframe y delega la lógica en `TonetApp`.
//! Más información: https://usetonet.com

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod network;
mod parser;
mod renderer;
mod ui;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Tonet")
            .with_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Tonet",
        native_options,
        Box::new(|cc| Ok(Box::new(app::TonetApp::new(cc)))),
    )
}
