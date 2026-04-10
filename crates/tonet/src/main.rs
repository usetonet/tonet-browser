//! Tonet desktop entry point — minimal from-scratch browser (MVP).
//! https://usetonet.com

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod i18n;
mod network;
mod parser;
mod renderer;
mod settings;
mod ui;
mod update;

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
