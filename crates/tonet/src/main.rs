//! Tonet desktop entry point — minimal from-scratch browser (MVP).
//! https://usetonet.com

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod branding;
mod browser_log;
mod chrome;
mod css_resolve;
#[allow(dead_code, unused_imports)]
#[path = "../../tonet-engine/src/css/mod.rs"]
mod css;
mod document_url;
#[allow(dead_code, unused_imports)]
#[path = "../../tonet-engine/src/html/mod.rs"]
mod html;
mod i18n;
mod internal_pages;
mod limits;
mod network;
mod new_tab;
mod parser;
mod policy;
mod renderer;
mod session_snapshot;
mod servo_engine;
mod settings;
mod shortcut_catalog;
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
    // Unify rustls 0.23 crypto for Tonet (reqwest) and Servo (servo-net): both use aws-lc-rs only.
    // Must run before any thread (e.g. Servo ResourceManager) builds a TLS client.
    static RUSTLS_CRYPTO: std::sync::Once = std::sync::Once::new();
    RUSTLS_CRYPTO.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });

    let mut viewport = egui::ViewportBuilder::default()
        .with_app_id("tonet")
        .with_title("Tonet")
        .with_inner_size([960.0, 640.0])
        .with_decorations(!window_chrome::integrated_title_chrome());
    if window_chrome::integrated_title_chrome() {
        // Compact minimum when using integrated title chrome (narrow layout).
        viewport = viewport.with_min_inner_size([340.0, 200.0]);
    }
    if let Some(icon) = window_icon_from_svg() {
        viewport = viewport.with_icon(icon);
    }

    let native_options = {
        #[cfg(all(feature = "servo-engine", windows))]
        {
            eframe::NativeOptions {
                viewport,
                // Slint-style Servo uses surfman GL in-process; egui uses wgpu (shared device interop is future work).
                renderer: eframe::Renderer::Wgpu,
                ..Default::default()
            }
        }
        #[cfg(not(all(feature = "servo-engine", windows)))]
        {
            eframe::NativeOptions {
                viewport,
                ..Default::default()
            }
        }
    };

    eframe::run_native(
        "Tonet",
        native_options,
        Box::new(|cc| Ok(Box::new(app::TonetApp::new(cc)))),
    )
}
