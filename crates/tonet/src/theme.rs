//! Tonet chrome palette and spacing. **Color** values follow the active [`UiTheme`](crate::settings::UiTheme)
//! (set via [`set_active_ui_theme`] each frame from persisted settings).

use std::cell::Cell;

use egui::Color32;

use crate::settings::UiTheme;

thread_local! {
    static ACTIVE_UI_THEME: Cell<UiTheme> = Cell::new(UiTheme::Dark);
}

/// Sync thread-local palette before reading color functions (called from the main UI each frame).
#[inline]
pub fn set_active_ui_theme(theme: UiTheme) {
    ACTIVE_UI_THEME.with(|c| c.set(theme));
}

#[inline]
pub fn active_ui_theme() -> UiTheme {
    ACTIVE_UI_THEME.with(|c| c.get())
}

#[inline]
fn pick(dark: Color32, light: Color32) -> Color32 {
    match active_ui_theme() {
        UiTheme::Dark => dark,
        UiTheme::Light => light,
    }
}

// ── Spacing grid ────────────────────────────────────────────────────
pub const SP: f32 = 4.0;
pub const SP2: f32 = 8.0;
pub const SP3: f32 = 12.0;

// ── Chrome button hit-area / rounding ───────────────────────────────
pub const CHROME_BTN: f32 = 36.0;
pub const CHROME_BTN_ROUNDING: f32 = 7.0;

/// Servo viewport right gutter (scrollbar track + arrows).
pub const SERVO_SCROLL_GUTTER: f32 = 16.0;
pub const SERVO_SCROLL_ARROW_H: f32 = 11.0;

// ── Chrome / content ────────────────────────────────────────────────
pub fn chrome_bg() -> Color32 {
    pick(
        Color32::from_rgb(36, 38, 44),
        Color32::from_rgb(232, 234, 240),
    )
}

pub fn content_bg() -> Color32 {
    pick(
        Color32::from_rgb(32, 33, 38),
        Color32::from_rgb(250, 250, 252),
    )
}

/// Right gutter beside the Servo viewport (decorative scrollbar track).
pub fn servo_scroll_gutter_fill() -> Color32 {
    pick(
        Color32::from_rgb(40, 42, 48),
        Color32::from_rgb(220, 222, 228),
    )
}

pub fn servo_scroll_thumb() -> Color32 {
    pick(
        Color32::from_rgb(118, 124, 138),
        Color32::from_rgb(140, 145, 158),
    )
}

pub fn servo_scroll_arrow() -> Color32 {
    pick(
        Color32::from_rgb(88, 92, 104),
        Color32::from_rgb(120, 125, 138),
    )
}

// ── Developer tools ─────────────────────────────────────────────────
pub fn devtools_bg() -> Color32 {
    pick(
        Color32::from_rgb(30, 31, 36),
        Color32::from_rgb(242, 243, 247),
    )
}

pub fn devtools_tab_active() -> Color32 {
    pick(
        Color32::from_rgb(42, 44, 52),
        Color32::from_rgb(228, 230, 238),
    )
}

pub fn devtools_tab_text_active() -> Color32 {
    pick(
        Color32::from_rgb(230, 232, 240),
        Color32::from_rgb(28, 30, 36),
    )
}

pub fn devtools_tab_text_idle() -> Color32 {
    pick(
        Color32::from_rgb(150, 154, 168),
        Color32::from_rgb(90, 95, 108),
    )
}

pub fn devtools_toolbar_icon() -> Color32 {
    pick(
        Color32::from_rgb(190, 194, 208),
        Color32::from_rgb(70, 75, 88),
    )
}

pub fn devtools_split_handle() -> Color32 {
    pick(
        Color32::from_rgb(55, 58, 68),
        Color32::from_rgb(200, 204, 214),
    )
}

// ── Tab strip ───────────────────────────────────────────────────────
pub fn tab_idle() -> Color32 {
    Color32::TRANSPARENT
}

pub fn tab_hover() -> Color32 {
    pick(
        Color32::from_rgb(43, 46, 52),
        Color32::from_rgb(210, 214, 222),
    )
}

pub fn tab_selected() -> Color32 {
    pick(
        Color32::from_rgb(50, 53, 60),
        Color32::from_rgb(198, 202, 212),
    )
}

pub fn tab_text_muted() -> Color32 {
    pick(
        Color32::from_rgb(155, 158, 168),
        Color32::from_rgb(95, 100, 112),
    )
}

pub fn tab_text() -> Color32 {
    pick(
        Color32::from_rgb(240, 241, 246),
        Color32::from_rgb(28, 30, 36),
    )
}

pub fn separator() -> Color32 {
    pick(
        Color32::from_rgb(52, 55, 62),
        Color32::from_rgb(190, 194, 204),
    )
}

// ── Navigation buttons ──────────────────────────────────────────────
pub fn nav_glyph() -> Color32 {
    pick(
        Color32::from_rgb(220, 222, 230),
        Color32::from_rgb(55, 58, 66),
    )
}

pub fn nav_glyph_disabled() -> Color32 {
    pick(
        Color32::from_rgb(100, 103, 118),
        Color32::from_rgb(160, 165, 175),
    )
}

// ── Omnibox pill ────────────────────────────────────────────────────
pub fn omnibox_fill() -> Color32 {
    pick(
        Color32::from_rgb(26, 28, 34),
        Color32::from_rgb(255, 255, 255),
    )
}

pub fn omnibox_stroke() -> Color32 {
    pick(
        Color32::from_rgb(50, 53, 62),
        Color32::from_rgb(180, 184, 194),
    )
}

pub fn omnibox_text() -> Color32 {
    pick(
        Color32::from_rgb(220, 222, 228),
        Color32::from_rgb(28, 30, 36),
    )
}

pub fn chip() -> Color32 {
    pick(
        Color32::from_rgb(170, 173, 185),
        Color32::from_rgb(85, 90, 102),
    )
}

pub fn tool_icon() -> Color32 {
    pick(
        Color32::from_rgb(200, 203, 215),
        Color32::from_rgb(70, 75, 88),
    )
}

// ── Accent ──────────────────────────────────────────────────────────
pub fn accent() -> Color32 {
    pick(
        Color32::from_rgb(132, 190, 255),
        Color32::from_rgb(50, 110, 200),
    )
}

pub fn primary_btn() -> Color32 {
    pick(
        Color32::from_rgb(80, 138, 224),
        Color32::from_rgb(60, 120, 210),
    )
}

// ── Caption (window buttons) ────────────────────────────────────────
pub fn caption_glyph() -> Color32 {
    pick(
        Color32::from_rgb(210, 212, 220),
        Color32::from_rgb(55, 58, 66),
    )
}

pub fn caption_close() -> Color32 {
    pick(
        Color32::from_rgb(218, 102, 108),
        Color32::from_rgb(200, 60, 70),
    )
}

pub fn caption_close_hover() -> Color32 {
    pick(
        Color32::from_rgb(232, 17, 35),
        Color32::from_rgb(220, 40, 50),
    )
}

// ── Semantic panels ─────────────────────────────────────────────────
pub fn error_bg() -> Color32 {
    pick(
        Color32::from_rgb(58, 38, 40),
        Color32::from_rgb(255, 235, 238),
    )
}

pub fn error_stroke() -> Color32 {
    pick(
        Color32::from_rgb(100, 62, 66),
        Color32::from_rgb(220, 160, 168),
    )
}

pub fn error_title() -> Color32 {
    pick(
        Color32::from_rgb(255, 205, 205),
        Color32::from_rgb(140, 40, 50),
    )
}

pub fn error_body() -> Color32 {
    pick(
        Color32::from_rgb(235, 210, 210),
        Color32::from_rgb(90, 45, 50),
    )
}

pub fn loading_muted() -> Color32 {
    pick(
        Color32::from_rgb(145, 148, 162),
        Color32::from_rgb(100, 105, 118),
    )
}

pub fn update_banner_bg() -> Color32 {
    pick(
        Color32::from_rgb(34, 50, 78),
        Color32::from_rgb(220, 235, 255),
    )
}

pub fn update_banner_stroke() -> Color32 {
    pick(
        Color32::from_rgb(72, 110, 172),
        Color32::from_rgb(120, 170, 220),
    )
}

pub fn update_accent_label() -> Color32 {
    pick(
        Color32::from_rgb(190, 215, 250),
        Color32::from_rgb(40, 90, 160),
    )
}

// ── Settings window ─────────────────────────────────────────────────
pub fn settings_window_bg() -> Color32 {
    pick(
        Color32::from_rgb(42, 44, 52),
        Color32::from_rgb(245, 246, 250),
    )
}

pub fn settings_heading() -> Color32 {
    pick(
        Color32::from_rgb(248, 248, 250),
        Color32::from_rgb(28, 30, 36),
    )
}

pub fn settings_status_bg() -> Color32 {
    pick(
        Color32::from_rgb(34, 36, 42),
        Color32::from_rgb(230, 232, 238),
    )
}

// ── Page body (renderer) ────────────────────────────────────────────
pub fn body_text() -> Color32 {
    pick(
        Color32::from_rgb(220, 222, 230),
        Color32::from_rgb(28, 30, 38),
    )
}

pub fn page_title() -> Color32 {
    pick(
        Color32::from_rgb(108, 168, 245),
        Color32::from_rgb(40, 100, 190),
    )
}

pub fn link() -> Color32 {
    pick(
        Color32::from_rgb(130, 188, 255),
        Color32::from_rgb(30, 100, 200),
    )
}

// ── New Tab page (card grid) ───────────────────────────────────────
pub fn nt_card_bg() -> Color32 {
    pick(
        Color32::from_rgb(38, 40, 48),
        Color32::from_rgb(240, 241, 246),
    )
}

pub fn nt_card_stroke() -> Color32 {
    pick(
        Color32::from_rgb(52, 55, 62),
        Color32::from_rgb(200, 204, 214),
    )
}

pub fn nt_tile_bg() -> Color32 {
    pick(
        Color32::from_rgb(48, 50, 58),
        Color32::from_rgb(228, 230, 236),
    )
}

pub fn nt_tile_hover() -> Color32 {
    pick(
        Color32::from_rgb(58, 61, 70),
        Color32::from_rgb(216, 220, 230),
    )
}

pub fn nt_tile_disabled() -> Color32 {
    pick(
        Color32::from_rgb(42, 44, 50),
        Color32::from_rgb(210, 212, 220),
    )
}

pub fn nt_search_bg() -> Color32 {
    pick(
        Color32::from_rgb(30, 32, 38),
        Color32::from_rgb(255, 255, 255),
    )
}

pub fn nt_search_stroke() -> Color32 {
    pick(
        Color32::from_rgb(60, 63, 72),
        Color32::from_rgb(190, 194, 204),
    )
}

pub fn nt_label_muted() -> Color32 {
    pick(
        Color32::from_rgb(155, 158, 168),
        Color32::from_rgb(100, 105, 118),
    )
}

pub fn nt_tile_icon_bg() -> Color32 {
    pick(
        Color32::from_rgb(58, 61, 70),
        Color32::from_rgb(200, 204, 214),
    )
}

pub fn nt_tile_icon_fg() -> Color32 {
    pick(
        Color32::from_rgb(200, 203, 215),
        Color32::from_rgb(55, 58, 66),
    )
}
