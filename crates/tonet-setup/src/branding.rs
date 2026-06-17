//! Tonet mark for the setup window (same SVG as the landing site).

pub const TONET_LOGO_URI: &str = "bytes://tonet-setup-logo.svg";

pub static TONET_SVG: &[u8] = include_bytes!("../../../web/landing/public/tonet.svg");

/// Windows icon copied beside `tonet.exe` for shortcuts and Add/Remove Programs.
pub static APP_ICO: &[u8] = include_bytes!("../../tonet/windows/app.ico");

/// Taskbar / window icon (Tonet mark, not the default egui "e").
pub fn window_icon() -> Option<egui::IconData> {
    let img = egui_extras::image::load_svg_bytes_with_size(
        TONET_SVG,
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

/// Progress bar + accent (Tonet blue, Chrome-style thin bar).
pub fn accent() -> egui::Color32 {
    egui::Color32::from_rgb(80, 138, 224)
}

pub fn track() -> egui::Color32 {
    egui::Color32::from_rgb(220, 222, 228)
}

pub fn body_text() -> egui::Color32 {
    egui::Color32::from_rgb(60, 64, 72)
}

pub fn wordmark() -> egui::Color32 {
    egui::Color32::from_rgb(120, 124, 132)
}
