//! New Tab page: Tonet logo, search bar, and shortcut grid (placeholder).

use egui::{Align, Align2, Color32, FontId, Layout, RichText, Sense, Stroke, Ui, Vec2};

use crate::branding;
use crate::i18n::{self, Locale};
use crate::theme;

const CARD_BG: Color32 = Color32::from_rgb(38, 40, 48);
const CARD_STROKE: Color32 = Color32::from_rgb(52, 55, 62);
const TILE_BG: Color32 = Color32::from_rgb(48, 50, 58);
const TILE_HOVER: Color32 = Color32::from_rgb(58, 61, 70);
const SEARCH_BG: Color32 = Color32::from_rgb(30, 32, 38);
const SEARCH_STROKE: Color32 = Color32::from_rgb(60, 63, 72);
const LABEL_MUTED: Color32 = Color32::from_rgb(155, 158, 168);

struct Shortcut {
    icon: &'static str,
    label: &'static str,
    url: &'static str,
}

const SHORTCUTS: &[Shortcut] = &[
    Shortcut { icon: "𝐓", label: "Tonet Home", url: "https://usetonet.com" },
    Shortcut { icon: "⊙", label: "GitHub", url: "https://github.com" },
    Shortcut { icon: "G", label: "Google", url: "https://google.com" },
    Shortcut { icon: "🛡", label: "Brave Search", url: "https://search.brave.com" },
    Shortcut { icon: "⚙", label: "Tonet settings", url: "tonet://settings" },
    Shortcut { icon: "⤓", label: "Downloads", url: "tonet://downloads" },
    Shortcut { icon: "🕐", label: "History", url: "tonet://history" },
    Shortcut { icon: "e", label: "egui docs", url: "https://docs.rs/egui" },
    Shortcut { icon: "🦀", label: "rust-lang.org", url: "https://rust-lang.org" },
    Shortcut { icon: "◉", label: "openai.com", url: "https://openai.com" },
    Shortcut { icon: "W", label: "wikipedia.org", url: "https://wikipedia.org" },
    Shortcut { icon: "+", label: "Add shortcut", url: "" },
];

pub struct NewTabAction {
    pub navigate_to: Option<String>,
}

pub fn show_new_tab_page(
    ui: &mut Ui,
    loc: Locale,
    url_input: &mut String,
) -> NewTabAction {
    let mut action = NewTabAction { navigate_to: None };

    ui.with_layout(Layout::top_down(Align::Center), |ui| {
        let avail_h = ui.available_height();
        ui.add_space((avail_h * 0.08).max(20.0));

        // ── Tonet Logo ──────────────────────────────────────────
        ui.add(
            egui::Image::from_uri(branding::TONET_LOGO_URI)
                .max_size(Vec2::splat(72.0))
                .rounding(8.0),
        );

        ui.add_space(24.0);

        // ── Search Bar ──────────────────────────────────────────
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
                            ui.label(
                                RichText::new("🔍").size(14.0).color(LABEL_MUTED),
                            );
                            ui.label(
                                RichText::new("🌐").size(14.0).color(LABEL_MUTED),
                            );

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

                            ui.label(
                                RichText::new("🎤").size(14.0).color(LABEL_MUTED),
                            );
                        });
                    });
            },
        );

        ui.add_space(32.0);

        // ── Shortcut Grid ───────────────────────────────────────
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
                        let cols = 5;
                        let tile_size = 80.0_f32;
                        let gap = 12.0_f32;

                        for row in SHORTCUTS.chunks(cols) {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = gap;
                                let row_w = cols as f32 * tile_size
                                    + (cols as f32 - 1.0) * gap;
                                let offset =
                                    ((ui.available_width() - row_w) / 2.0).max(0.0);
                                ui.add_space(offset);

                                for shortcut in row {
                                    draw_shortcut_tile(
                                        ui,
                                        shortcut,
                                        tile_size,
                                        &mut action,
                                    );
                                }
                            });
                            ui.add_space(gap);
                        }
                    });
            },
        );
    });

    action
}

fn draw_shortcut_tile(
    ui: &mut Ui,
    shortcut: &Shortcut,
    size: f32,
    action: &mut NewTabAction,
) {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());

    let bg = if resp.hovered() { TILE_HOVER } else { TILE_BG };
    ui.painter()
        .rect_filled(rect, 12.0, bg);

    let icon_rect_size = 36.0;
    let icon_center = rect.center_top() + egui::vec2(0.0, 8.0 + icon_rect_size / 2.0);
    ui.painter().rect_filled(
        egui::Rect::from_center_size(
            icon_center,
            Vec2::splat(icon_rect_size),
        ),
        8.0,
        Color32::from_rgb(58, 61, 70),
    );
    ui.painter().text(
        icon_center,
        Align2::CENTER_CENTER,
        shortcut.icon,
        FontId::proportional(18.0),
        Color32::from_rgb(200, 203, 215),
    );

    let label_pos = rect.center_top() + egui::vec2(0.0, 8.0 + icon_rect_size + 8.0);
    ui.painter().text(
        label_pos,
        Align2::CENTER_TOP,
        shortcut.label,
        FontId::proportional(10.5),
        LABEL_MUTED,
    );

    if resp.clicked() && !shortcut.url.is_empty() {
        action.navigate_to = Some(shortcut.url.to_string());
    }

    if !shortcut.url.is_empty() {
        resp.on_hover_text(shortcut.url);
    }
}
