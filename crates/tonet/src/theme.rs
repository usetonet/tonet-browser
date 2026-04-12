//! Tonet chrome palette — **high contrast** between the top chrome and the page
//! so the new UI reads clearly (issue #27). Neutrals lean blue-gray.

use egui::Color32;

// --- Page shell (darker than chrome so content “recedes”) ---
pub const CONTENT_BG: Color32 = Color32::from_rgb(14, 15, 19);
/// Top panel fill (visible in gaps / margins under child widgets).
pub const PANEL_FILL: Color32 = Color32::from_rgb(17, 18, 24);

// --- Tab strip (full-width rail behind floating pills) ---
pub const STRIP_BG: Color32 = Color32::from_rgb(17, 18, 24);
pub const STRIP_STROKE: Color32 = Color32::from_rgb(44, 48, 60);
/// Floating tab pills — clearly lighter than the rail.
pub const TAB_IDLE: Color32 = Color32::from_rgb(48, 50, 60);
pub const TAB_SELECTED: Color32 = Color32::from_rgb(58, 62, 78);
pub const TAB_SELECTED_STROKE: Color32 = Color32::from_rgb(96, 158, 245);
/// Hairline border on unselected pills so they read on the dark rail.
pub const TAB_PILL_STROKE_IDLE: Color32 = Color32::from_rgb(58, 60, 72);
pub const TAB_TEXT_MUTED: Color32 = Color32::from_rgb(168, 170, 182);
pub const TAB_TEXT: Color32 = Color32::from_rgb(248, 248, 252);

// --- “Address card”: brand + toolbar (elevated block under tabs) ---
pub const CHROME_CARD: Color32 = Color32::from_rgb(30, 32, 40);
pub const CHROME_CARD_STROKE: Color32 = Color32::from_rgb(56, 60, 76);

/// Omnibox sits on the card; slightly inset dark field with cool border.
pub const OMNIBOX_FILL: Color32 = Color32::from_rgb(18, 19, 26);
pub const OMNIBOX_STROKE: Color32 = Color32::from_rgb(64, 88, 132);
pub const CHIP: Color32 = Color32::from_rgb(155, 158, 172);
pub const NAV_BTN_FILL: Color32 = Color32::from_rgb(44, 46, 58);
pub const PRIMARY_BTN: Color32 = Color32::from_rgb(64, 132, 228);

// --- Accents (brand row, in-page links) ---
pub const ACCENT: Color32 = Color32::from_rgb(118, 182, 255);

// --- Window caption (integrated chrome) ---
pub const CAPTION_BTN_FILL: Color32 = Color32::from_rgb(42, 44, 54);
pub const CAPTION_GLYPH: Color32 = Color32::from_rgb(232, 232, 238);
pub const CAPTION_CLOSE: Color32 = Color32::from_rgb(218, 102, 108);

// --- Semantic panels ---
pub const ERROR_BG: Color32 = Color32::from_rgb(48, 34, 36);
pub const ERROR_STROKE: Color32 = Color32::from_rgb(88, 62, 66);
pub const ERROR_TITLE: Color32 = Color32::from_rgb(255, 205, 205);
pub const ERROR_BODY: Color32 = Color32::from_rgb(235, 210, 210);

pub const LOADING_MUTED: Color32 = Color32::from_rgb(138, 140, 154);

pub const UPDATE_BANNER_BG: Color32 = Color32::from_rgb(28, 44, 72);
pub const UPDATE_BANNER_STROKE: Color32 = Color32::from_rgb(68, 104, 168);
pub const UPDATE_ACCENT_LABEL: Color32 = Color32::from_rgb(185, 210, 248);

// --- Settings window ---
pub const SETTINGS_WINDOW_BG: Color32 = Color32::from_rgb(34, 36, 44);
pub const SETTINGS_HEADING: Color32 = Color32::from_rgb(248, 248, 250);
pub const SETTINGS_STATUS_BG: Color32 = Color32::from_rgb(24, 25, 32);

// --- Page body (renderer) ---
pub const BODY_TEXT: Color32 = Color32::from_rgb(220, 222, 230);
pub const PAGE_TITLE: Color32 = Color32::from_rgb(108, 168, 245);
pub const LINK: Color32 = Color32::from_rgb(130, 188, 255);
