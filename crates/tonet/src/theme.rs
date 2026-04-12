//! Calm dark palette and typography-adjacent tuning for Tonet (issue #27).
//! Surfaces lean neutral-warm; accents are restrained blues.

use egui::Color32;

// --- App shell (matches egui `Visuals` tuned in `TonetApp::new`) ---
pub const CONTENT_BG: Color32 = Color32::from_rgb(24, 24, 26);
pub const PANEL_FILL: Color32 = Color32::from_rgb(26, 26, 28);

// --- Tab strip ---
pub const STRIP_BG: Color32 = Color32::from_rgb(30, 30, 32);
pub const STRIP_STROKE: Color32 = Color32::from_rgb(48, 48, 52);
pub const TAB_IDLE: Color32 = Color32::from_rgb(38, 38, 42);
pub const TAB_SELECTED: Color32 = Color32::from_rgb(50, 50, 56);
pub const TAB_SELECTED_STROKE: Color32 = Color32::from_rgb(72, 118, 188);
pub const TAB_TEXT_MUTED: Color32 = Color32::from_rgb(175, 175, 182);
pub const TAB_TEXT: Color32 = Color32::from_rgb(236, 236, 240);

// --- Toolbar & omnibox ---
pub const TOOLBAR_BG: Color32 = Color32::from_rgb(36, 36, 40);
pub const OMNIBOX_FILL: Color32 = Color32::from_rgb(28, 28, 32);
pub const OMNIBOX_STROKE: Color32 = Color32::from_rgb(58, 58, 64);
pub const CHIP: Color32 = Color32::from_rgb(148, 148, 156);
pub const NAV_BTN_FILL: Color32 = Color32::from_rgb(44, 44, 50);
pub const PRIMARY_BTN: Color32 = Color32::from_rgb(72, 128, 210);

// --- Accents (brand row, in-page links) ---
pub const ACCENT: Color32 = Color32::from_rgb(110, 168, 240);

// --- Window caption (integrated chrome) ---
pub const CAPTION_BTN_FILL: Color32 = Color32::from_rgb(46, 46, 52);
pub const CAPTION_GLYPH: Color32 = Color32::from_rgb(230, 230, 235);
pub const CAPTION_CLOSE: Color32 = Color32::from_rgb(210, 96, 102);

// --- Semantic panels ---
pub const ERROR_BG: Color32 = Color32::from_rgb(48, 34, 36);
pub const ERROR_STROKE: Color32 = Color32::from_rgb(88, 62, 66);
pub const ERROR_TITLE: Color32 = Color32::from_rgb(255, 205, 205);
pub const ERROR_BODY: Color32 = Color32::from_rgb(235, 210, 210);

pub const LOADING_MUTED: Color32 = Color32::from_rgb(138, 138, 148);

pub const UPDATE_BANNER_BG: Color32 = Color32::from_rgb(32, 48, 72);
pub const UPDATE_BANNER_STROKE: Color32 = Color32::from_rgb(72, 108, 168);
pub const UPDATE_ACCENT_LABEL: Color32 = Color32::from_rgb(185, 210, 248);

// --- Settings window ---
pub const SETTINGS_WINDOW_BG: Color32 = Color32::from_rgb(38, 38, 42);
pub const SETTINGS_HEADING: Color32 = Color32::from_rgb(248, 248, 250);
pub const SETTINGS_STATUS_BG: Color32 = Color32::from_rgb(32, 32, 36);

// --- Page body (renderer) ---
pub const BODY_TEXT: Color32 = Color32::from_rgb(218, 218, 224);
pub const PAGE_TITLE: Color32 = Color32::from_rgb(95, 145, 210);
pub const LINK: Color32 = Color32::from_rgb(118, 178, 245);
