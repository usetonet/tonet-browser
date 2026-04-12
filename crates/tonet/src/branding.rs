//! Tonet mark: same SVG as `web/landing/public/tonet.svg` (favicon / landing logo).

/// URI registered with [`egui::Context::include_bytes`] for the in-app logo.
pub const TONET_LOGO_URI: &str = "bytes://tonet-logo.svg";

/// Raw SVG bytes (shared with the marketing site).
pub static TONET_SVG: &[u8] = include_bytes!("../../../web/landing/public/tonet.svg");
