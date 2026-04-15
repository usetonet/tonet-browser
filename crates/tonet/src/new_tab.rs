//! New Tab page: Tonet logo, search bar, and shortcut grid (shortcuts persist in settings).

use egui::{Align, Align2, Color32, Context, FontId, Layout, RichText, Sense, Stroke, TextEdit, Ui, Vec2};

use crate::branding;
use crate::i18n::{self, Locale};
use crate::settings::{
    is_allowed_new_tab_url, AppSettings, NewTabShortcut, NEW_TAB_SHORTCUTS_MAX,
};
use crate::theme;

const CARD_BG: Color32 = Color32::from_rgb(38, 40, 48);
const CARD_STROKE: Color32 = Color32::from_rgb(52, 55, 62);
const TILE_BG: Color32 = Color32::from_rgb(48, 50, 58);
const TILE_HOVER: Color32 = Color32::from_rgb(58, 61, 70);
const TILE_DISABLED: Color32 = Color32::from_rgb(42, 44, 50);
const SEARCH_BG: Color32 = Color32::from_rgb(30, 32, 38);
const SEARCH_STROKE: Color32 = Color32::from_rgb(60, 63, 72);
const LABEL_MUTED: Color32 = Color32::from_rgb(155, 158, 168);

const GRID_COLS: usize = 5;
const TILE_SIZE: f32 = 80.0;
const TILE_GAP: f32 = 12.0;

#[derive(Default)]
pub struct NewTabAddState {
    pub open: bool,
    pub icon: String,
    pub label: String,
    pub url: String,
    pub error: String,
}

pub struct NewTabAction {
    pub navigate_to: Option<String>,
    /// Settings JSON should be written (shortcuts added, removed, or edited).
    pub need_save: bool,
}

impl NewTabAction {
    fn none() -> Self {
        Self {
            navigate_to: None,
            need_save: false,
        }
    }
}

fn normalized_shortcut(icon: String, label: String, url: String) -> NewTabShortcut {
    let url = url.trim().to_string();
    let mut icon = icon.trim().to_string();
    if icon.is_empty() {
        icon = "🔗".to_string();
    }
    if icon.chars().count() > 8 {
        icon = icon.chars().take(8).collect();
    }
    let mut label = label.trim().to_string();
    if label.is_empty() {
        label = url::Url::parse(&url)
            .ok()
            .and_then(|u| u.host_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "Link".to_string());
    }
    if label.chars().count() > 40 {
        label = label.chars().take(40).collect();
    }
    NewTabShortcut { icon, label, url }
}

pub fn show_new_tab_page(
    ui: &mut Ui,
    ctx: &Context,
    loc: Locale,
    url_input: &mut String,
    settings: &mut AppSettings,
    add: &mut NewTabAddState,
) -> NewTabAction {
    let mut action = NewTabAction::none();
    let shortcuts = &mut settings.new_tab_shortcuts;

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        let avail_h = ui.available_height();
        ui.add_space((avail_h * 0.08).max(20.0));

        ui.add(
            egui::Image::from_uri(branding::TONET_LOGO_URI)
                .max_size(Vec2::splat(72.0))
                .rounding(8.0),
        );

        ui.add_space(24.0);

        let search_w = (ui.available_width() * 0.45).clamp(320.0, 560.0);
        ui.allocate_ui_with_layout(
            Vec2::new(search_w, 44.0),
            Layout::left_to_right(Align::Center),
            |ui| {
                egui::Frame::none()
                    .fill(SEARCH_BG)
                    .stroke(Stroke::new(1.0, SEARCH_STROKE))
                    .rounding(22.0)
                    .inner_margin(egui::Margin::symmetric(16.0, 8.0))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 8.0;
                            ui.label(RichText::new("🔍").size(14.0).color(LABEL_MUTED));
                            ui.label(RichText::new("🌐").size(14.0).color(LABEL_MUTED));

                            let te = egui::TextEdit::singleline(url_input)
                                .id(egui::Id::new("new_tab_search"))
                                .frame(false)
                                .text_color(theme::OMNIBOX_TEXT)
                                .hint_text(i18n::new_tab_search_hint(loc))
                                .desired_rows(1)
                                .desired_width(ui.available_width() - 24.0);
                            let output = te.show(ui);

                            if output.response.lost_focus()
                                && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                && !url_input.trim().is_empty()
                            {
                                action.navigate_to = Some(url_input.clone());
                            }

                            ui.label(RichText::new("🎤").size(14.0).color(LABEL_MUTED));
                        });
                    });
            },
        );

        ui.add_space(32.0);

        let card_w = (ui.available_width() * 0.45).clamp(340.0, 580.0);
        ui.allocate_ui_with_layout(
            Vec2::new(card_w, 0.0),
            Layout::top_down(Align::Center),
            |ui| {
                egui::Frame::none()
                    .fill(CARD_BG)
                    .stroke(Stroke::new(1.0, CARD_STROKE))
                    .rounding(16.0)
                    .inner_margin(egui::Margin::symmetric(20.0, 20.0))
                    .show(ui, |ui| {
                        render_shortcut_grid(ui, loc, shortcuts, add, &mut action);
                    });
            },
        );
    });

    show_add_shortcut_window(ctx, loc, shortcuts, add, &mut action);

    action
}

fn render_shortcut_grid(
    ui: &mut Ui,
    loc: Locale,
    shortcuts: &mut Vec<NewTabShortcut>,
    add: &mut NewTabAddState,
    action: &mut NewTabAction,
) {
    let n = shortcuts.len();
    let can_add = n < NEW_TAB_SHORTCUTS_MAX;
    let cols = GRID_COLS;

    if n == 0 {
        ui.horizontal(|ui| {
            center_row_offset(ui, 1);
            draw_add_tile(ui, loc, TILE_SIZE, can_add, add);
        });
        ui.add_space(TILE_GAP);
        return;
    }

    let full_rows = n / cols;
    for row in 0..full_rows {
        let start = row * cols;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = TILE_GAP;
            center_row_offset(ui, cols);
            for j in 0..cols {
                let idx = start + j;
                let tile = shortcuts[idx].clone();
                draw_link_tile(ui, loc, tile, TILE_SIZE, idx, shortcuts, action);
            }
        });
        ui.add_space(TILE_GAP);
    }

    let rem = n % cols;
    if rem > 0 {
        let start = full_rows * cols;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = TILE_GAP;
            let tiles_in_row = (rem + if can_add { 1 } else { 0 }).min(cols);
            center_row_offset(ui, tiles_in_row);
            for j in 0..rem {
                let idx = start + j;
                let tile = shortcuts[idx].clone();
                draw_link_tile(ui, loc, tile, TILE_SIZE, idx, shortcuts, action);
            }
            if can_add {
                draw_add_tile(ui, loc, TILE_SIZE, can_add, add);
            }
        });
        ui.add_space(TILE_GAP);
    } else if can_add {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = TILE_GAP;
            center_row_offset(ui, 1);
            draw_add_tile(ui, loc, TILE_SIZE, can_add, add);
        });
        ui.add_space(TILE_GAP);
    }
}

fn center_row_offset(ui: &mut Ui, tiles_in_row: usize) {
    let row_w = tiles_in_row as f32 * TILE_SIZE + (tiles_in_row.saturating_sub(1) as f32) * TILE_GAP;
    let offset = ((ui.available_width() - row_w) / 2.0).max(0.0);
    ui.add_space(offset);
}

fn draw_link_tile(
    ui: &mut Ui,
    loc: Locale,
    shortcut: NewTabShortcut,
    size: f32,
    index: usize,
    shortcuts: &mut Vec<NewTabShortcut>,
    action: &mut NewTabAction,
) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());

    let bg = if resp.hovered() { TILE_HOVER } else { TILE_BG };
    ui.painter().rect_filled(rect, 12.0, bg);

    let icon_rect_size = 36.0;
    let icon_center = rect.center_top() + egui::vec2(0.0, 8.0 + icon_rect_size / 2.0);
    ui.painter().rect_filled(
        egui::Rect::from_center_size(icon_center, Vec2::splat(icon_rect_size)),
        8.0,
        Color32::from_rgb(58, 61, 70),
    );
    ui.painter().text(
        icon_center,
        Align2::CENTER_CENTER,
        shortcut.icon.as_str(),
        FontId::proportional(18.0),
        Color32::from_rgb(200, 203, 215),
    );

    let label_pos = rect.center_top() + egui::vec2(0.0, 8.0 + icon_rect_size + 8.0);
    ui.painter().text(
        label_pos,
        Align2::CENTER_TOP,
        shortcut.label.as_str(),
        FontId::proportional(10.5),
        LABEL_MUTED,
    );

    if resp.clicked() && is_allowed_new_tab_url(&shortcut.url) {
        action.navigate_to = Some(shortcut.url.clone());
    }

    let resp = resp.on_hover_text(shortcut.url.as_str());
    resp.context_menu(|ui| {
        if shortcuts.len() > 1 && ui.button(i18n::new_tab_remove(loc)).clicked() {
            shortcuts.remove(index);
            action.need_save = true;
            ui.close_menu();
        }
    });
}

fn draw_add_tile(
    ui: &mut Ui,
    loc: Locale,
    size: f32,
    enabled: bool,
    add: &mut NewTabAddState,
) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    let bg = if !enabled {
        TILE_DISABLED
    } else if resp.hovered() {
        TILE_HOVER
    } else {
        TILE_BG
    };
    ui.painter().rect_filled(rect, 12.0, bg);

    let icon_rect_size = 36.0;
    let icon_center = rect.center_top() + egui::vec2(0.0, 8.0 + icon_rect_size / 2.0);
    ui.painter().rect_filled(
        egui::Rect::from_center_size(icon_center, Vec2::splat(icon_rect_size)),
        8.0,
        Color32::from_rgb(58, 61, 70),
    );
    ui.painter().text(
        icon_center,
        Align2::CENTER_CENTER,
        "+",
        FontId::proportional(18.0),
        Color32::from_rgb(200, 203, 215),
    );

    let label = i18n::new_tab_add_tile_label(loc);
    let label_pos = rect.center_top() + egui::vec2(0.0, 8.0 + icon_rect_size + 8.0);
    ui.painter().text(
        label_pos,
        Align2::CENTER_TOP,
        label,
        FontId::proportional(10.5),
        LABEL_MUTED,
    );

    if enabled {
        if resp.clicked() {
            add.open = true;
            add.error.clear();
        }
        resp.on_hover_text(i18n::new_tab_add_tile_hint(loc));
    } else {
        resp.on_hover_text(i18n::new_tab_add_max_tiles(loc));
    }
}

fn show_add_shortcut_window(
    ctx: &Context,
    loc: Locale,
    shortcuts: &mut Vec<NewTabShortcut>,
    add: &mut NewTabAddState,
    action: &mut NewTabAction,
) {
    let mut open = add.open;
    if !open {
        return;
    }

    egui::Window::new(i18n::new_tab_add_title(loc))
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .open(&mut open)
        .show(ctx, |ui| {
            ui.set_min_width(320.0);
            ui.label(
                RichText::new(i18n::new_tab_add_intro(loc))
                    .small()
                    .color(theme::LOADING_MUTED),
            );
            ui.add_space(10.0);

            ui.label(RichText::new(i18n::new_tab_add_icon_label(loc)).strong());
            ui.add(TextEdit::singleline(&mut add.icon).desired_width(f32::INFINITY));
            ui.add_space(6.0);

            ui.label(RichText::new(i18n::new_tab_add_label_label(loc)).strong());
            ui.add(TextEdit::singleline(&mut add.label).desired_width(f32::INFINITY));
            ui.add_space(6.0);

            ui.label(RichText::new(i18n::new_tab_add_url_label(loc)).strong());
            ui.add(TextEdit::singleline(&mut add.url).desired_width(f32::INFINITY));

            if !add.error.is_empty() {
                ui.add_space(6.0);
                ui.label(
                    RichText::new(&add.error)
                        .small()
                        .color(Color32::from_rgb(220, 140, 100)),
                );
            }

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(i18n::new_tab_add_cancel(loc)).clicked() {
                    add.open = false;
                    add.error.clear();
                }
                if ui.button(i18n::new_tab_add_save(loc)).clicked() {
                    if shortcuts.len() >= NEW_TAB_SHORTCUTS_MAX {
                        add.error = i18n::new_tab_add_max_tiles(loc).to_string();
                    } else if !is_allowed_new_tab_url(&add.url) {
                        add.error = i18n::new_tab_add_url_invalid(loc).to_string();
                    } else {
                        let entry = normalized_shortcut(
                            std::mem::take(&mut add.icon),
                            std::mem::take(&mut add.label),
                            std::mem::take(&mut add.url),
                        );
                        shortcuts.push(entry);
                        action.need_save = true;
                        add.open = false;
                        add.error.clear();
                    }
                }
            });
        });

    add.open = open;
}
