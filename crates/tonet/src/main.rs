//! Tonet desktop entry point — minimal from-scratch browser (MVP).
//! https://usetonet.com

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod branding;
mod chrome;
mod i18n;
mod network;
mod new_tab;
mod parser;
mod renderer;
mod settings;
mod tab;
mod theme;
mod ui;
mod update;
mod window_chrome;
mod window_resize;

#[cfg(windows)]
mod platform_windows;

use eframe::egui;

fn window_icon_from_svg() -> Option<egui::IconData> {
    let img = egui_extras::image::load_svg_bytes_with_size(
        branding::TONET_SVG,
        Some(egui::SizeHint::Size(128, 128)),
    )
    .ok()?;
    let [w, h] = img.size;
    let mut rgba = Vec::with_capacity(w * h * 4);
    for c in img.pixels {
        rgba.extend_from_slice(&[c.r(), c.g(), c.b(), c.a()]);
    }
    Some(egui::IconData {
        rgba,
        width: w as u32,
        height: h as u32,
    })
}

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title("Tonet")
        .with_inner_size([960.0, 640.0])
        .with_decorations(!window_chrome::integrated_title_chrome());
    if window_chrome::integrated_title_chrome() {
        // Brave-like minimum: roughly phone width × chrome-only height.
        viewport = viewport.with_min_inner_size([340.0, 200.0]);
    }
    if let Some(icon) = window_icon_from_svg() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    eframe::run_native(
        "Tonet",
        native_options,
        Box::new(|cc| Ok(Box::new(app::TonetApp::new(cc)))),
    )
}
