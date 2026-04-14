//! Tonet chrome palette and spacing system.

use egui::Color32;

// ── Spacing grid ────────────────────────────────────────────────────
pub const SP: f32 = 4.0;
pub const SP2: f32 = 8.0;
pub const SP3: f32 = 12.0;

// ── Chrome button hit-area / rounding ───────────────────────────────
pub const CHROME_BTN: f32 = 32.0;
pub const CHROME_BTN_ROUNDING: f32 = 6.0;

// ── Single chrome background (tab strip + toolbar, no bands) ────────
pub const CHROME_BG: Color32 = Color32::from_rgb(36, 38, 44);

// ── Page content ────────────────────────────────────────────────────
pub const CONTENT_BG: Color32 = Color32::from_rgb(32, 33, 38);

// ── Tab strip ───────────────────────────────────────────────────────
pub const TAB_IDLE: Color32 = Color32::TRANSPARENT;
pub const TAB_HOVER: Color32 = Color32::from_rgb(43, 46, 52);
pub const TAB_SELECTED: Color32 = Color32::from_rgb(50, 53, 60);
pub const TAB_TEXT_MUTED: Color32 = Color32::from_rgb(155, 158, 168);
pub const TAB_TEXT: Color32 = Color32::from_rgb(240, 241, 246);

// ── 1px separator between tab strip and toolbar ─────────────────────
pub const SEPARATOR: Color32 = Color32::from_rgb(52, 55, 62);

// ── Navigation buttons ──────────────────────────────────────────────
pub const NAV_GLYPH: Color32 = Color32::from_rgb(220, 222, 230);
pub const NAV_GLYPH_DISABLED: Color32 = Color32::from_rgb(100, 103, 118);

// ── Omnibox pill ────────────────────────────────────────────────────
pub const OMNIBOX_FILL: Color32 = Color32::from_rgb(26, 28, 34);
pub const OMNIBOX_STROKE: Color32 = Color32::from_rgb(50, 53, 62);
pub const OMNIBOX_TEXT: Color32 = Color32::from_rgb(220, 222, 228);
pub const CHIP: Color32 = Color32::from_rgb(170, 173, 185);

// ── Toolbar icons ───────────────────────────────────────────────────
pub const TOOL_ICON: Color32 = Color32::from_rgb(200, 203, 215);


// ── Accent ──────────────────────────────────────────────────────────
pub const ACCENT: Color32 = Color32::from_rgb(132, 190, 255);
pub const PRIMARY_BTN: Color32 = Color32::from_rgb(80, 138, 224);

// ── Caption (window buttons) ────────────────────────────────────────
pub const CAPTION_GLYPH: Color32 = Color32::from_rgb(210, 212, 220);
pub const CAPTION_CLOSE: Color32 = Color32::from_rgb(218, 102, 108);
pub const CAPTION_CLOSE_HOVER: Color32 = Color32::from_rgb(232, 17, 35);

// ── Semantic panels ─────────────────────────────────────────────────
pub const ERROR_BG: Color32 = Color32::from_rgb(58, 38, 40);
pub const ERROR_STROKE: Color32 = Color32::from_rgb(100, 62, 66);
pub const ERROR_TITLE: Color32 = Color32::from_rgb(255, 205, 205);
pub const ERROR_BODY: Color32 = Color32::from_rgb(235, 210, 210);

pub const LOADING_MUTED: Color32 = Color32::from_rgb(145, 148, 162);

pub const UPDATE_BANNER_BG: Color32 = Color32::from_rgb(34, 50, 78);
pub const UPDATE_BANNER_STROKE: Color32 = Color32::from_rgb(72, 110, 172);
pub const UPDATE_ACCENT_LABEL: Color32 = Color32::from_rgb(190, 215, 250);

// ── Settings window ─────────────────────────────────────────────────
pub const SETTINGS_WINDOW_BG: Color32 = Color32::from_rgb(42, 44, 52);
pub const SETTINGS_HEADING: Color32 = Color32::from_rgb(248, 248, 250);
pub const SETTINGS_STATUS_BG: Color32 = Color32::from_rgb(34, 36, 42);

// ── Page body (renderer) ────────────────────────────────────────────
pub const BODY_TEXT: Color32 = Color32::from_rgb(220, 222, 230);
pub const PAGE_TITLE: Color32 = Color32::from_rgb(108, 168, 245);
pub const LINK: Color32 = Color32::from_rgb(130, 188, 255);
