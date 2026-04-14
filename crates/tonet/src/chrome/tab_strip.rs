//! Tab strip row -- Chrome/Brave-style: [favicon + title] tabs, "+" button, caption controls.
//!
//! This component draws NO background frame of its own; the parent TopPanel provides
//! the unified `CHROME_BG` fill.

use egui::{Align, Align2, Color32, Context, FontId, Layout, RichText, Sense, Shape, Ui, Vec2};

use crate::i18n::{self, Locale};
use crate::theme;

use super::caption::{apply_drag_or_maximize, show_window_caption_controls, CAPTION_BTN};

const TAB_ROW_H: f32 = 33.0;

#[derive(Default)]
pub struct TabBarResult {
    pub new_tab: bool,
    pub select_tab: Option<usize>,
    pub close_tab: Option<usize>,
}

const SHOW_CLOSE_THRESHOLD: f32 = 48.0;
const SHOW_TITLE_THRESHOLD: f32 = 56.0;

#[allow(clippy::too_many_arguments)]
fn draw_tab(
    ui: &mut Ui,
    index: usize,
    title: &str,
    favicon_uri: &str,
    selected: bool,
    can_close: bool,
    loc: Locale,
    out: &mut TabBarResult,
    max_w: f32,
) {
    let tab_bg = if selected {
        theme::TAB_SELECTED
    } else {
        theme::TAB_IDLE
    };
    let rounding = egui::Rounding {
        nw: 8.0,
        ne: 8.0,
        sw: 0.0,
        se: 0.0,
    };

    let show_title = max_w >= SHOW_TITLE_THRESHOLD;
    let show_close = can_close && max_w >= SHOW_CLOSE_THRESHOLD;
    let h_pad = if show_title { theme::SP2 } else { theme::SP };

    let hover_idx = ui.painter().add(Shape::Noop);

    let compact_mode = !show_title;

    let push_resp = ui.push_id(index as i32, |ui| {
        ui.set_max_width(max_w);

        let mut favicon_center = egui::Pos2::ZERO;

        let frame_resp = egui::Frame::none()
            .fill(tab_bg)
            .rounding(rounding)
            .inner_margin(egui::Margin::symmetric(h_pad, theme::SP))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = theme::SP;

                    if favicon_uri.is_empty() {
                        let (_, r) = ui.allocate_space(Vec2::splat(16.0));
                        favicon_center = r.center();
                    } else {
                        let resp = ui.add(
                            egui::Image::from_uri(favicon_uri)
                                .max_size(Vec2::splat(16.0))
                                .rounding(2.0),
                        );
                        favicon_center = resp.rect.center();
                    }

                    if show_title {
                        let text_color = if selected {
                            theme::TAB_TEXT
                        } else {
                            theme::TAB_TEXT_MUTED
                        };
                        let close_w = if show_close { 18.0 + theme::SP } else { 0.0 };
                        ui.scope(|ui| {
                            ui.set_max_width((ui.available_width() - close_w).max(0.0));
                            let resp = ui.add(
                                egui::Label::new(
                                    RichText::new(title).size(13.0).color(text_color),
                                )
                                .selectable(false)
                                .truncate()
                                .sense(Sense::click()),
                            );
                            if resp.clicked() {
                                out.select_tab = Some(index);
                            }
                        });
                    }

                    if show_close {
                        let close = ui.add_sized(
                            Vec2::splat(18.0),
                            egui::Button::new(
                                RichText::new("×")
                                    .size(14.0)
                                    .color(theme::TAB_TEXT_MUTED),
                            )
                            .frame(false),
                        );
                        if close
                            .on_hover_text(i18n::tab_close_tooltip(loc))
                            .clicked()
                        {
                            out.close_tab = Some(index);
                        }
                    }
                });
            });

        (frame_resp.response.rect, frame_resp.response.hovered(), favicon_center)
    });

    let (tab_rect, hovered, favicon_center) = push_resp.inner;

    if compact_mode && hovered {
        let hover_pos = ui.input(|i| i.pointer.hover_pos().unwrap_or_default());
        if tab_rect.contains(hover_pos) {
            if selected && can_close {
                ui.painter().rect_filled(
                    egui::Rect::from_center_size(favicon_center, Vec2::splat(16.0)),
                    2.0,
                    tab_bg,
                );
                ui.painter().text(
                    favicon_center,
                    Align2::CENTER_CENTER,
                    "×",
                    FontId::proportional(13.0),
                    Color32::from_rgb(200, 203, 215),
                );

                let close_rect = egui::Rect::from_center_size(favicon_center, Vec2::splat(16.0));
                let close_resp = ui.interact(close_rect, ui.id().with(("compact_close", index)), Sense::click());
                if close_resp.clicked() {
                    out.close_tab = Some(index);
                }
            } else {
                egui::show_tooltip(
                    ui.ctx(),
                    ui.layer_id(),
                    ui.id().with(("tab_tip", index)),
                    |ui| {
                        ui.label(title);
                    },
                );
            }
        }
    }

    let click_response = ui.interact(tab_rect, ui.id().with(("tab_click", index)), Sense::click());
    if click_response.clicked() {
        out.select_tab = Some(index);
    }

    if !selected && hovered {
        ui.painter().set(
            hover_idx,
            Shape::rect_filled(tab_rect, rounding, theme::TAB_HOVER),
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub fn show_tab_bar(
    ui: &mut Ui,
    ctx: &Context,
    loc: Locale,
    tab_titles: &[String],
    tab_favicons: &[String],
    active_index: usize,
    can_close_any: bool,
    integrated_caption: bool,
) -> TabBarResult {
    let mut out = TabBarResult::default();

    const DRAG_GAP: f32 = 4.0;
    let caption_w = CAPTION_BTN.x * 3.0;
    let right_chrome = if integrated_caption {
        DRAG_GAP + caption_w
    } else {
        0.0
    };

    ui.horizontal(|ui| {
        ui.set_height(TAB_ROW_H);
        ui.spacing_mut().item_spacing.x = 0.0;

        let total_w = ui.available_width();
        let tab_area_w = (total_w - right_chrome).max(64.0);
        let plus_w = 28.0 + theme::SP;
        let left_pad = theme::SP;
        let num = tab_titles.len().max(1) as f32;
        let gap_px = if num > 20.0 { 1.0 } else { 2.0 };
        let gaps = (tab_titles.len().saturating_sub(1) as f32) * gap_px;
        let tabs_avail = (tab_area_w - plus_w - left_pad - theme::SP).max(0.0);
        const MAX_TAB_W: f32 = 220.0;
        let per_tab_w = ((tabs_avail - gaps) / num).clamp(1.0, MAX_TAB_W);

        ui.allocate_ui_with_layout(
            Vec2::new(tab_area_w, TAB_ROW_H),
            Layout::left_to_right(Align::Center),
            |ui| {
                let clip = ui.max_rect();
                ui.set_clip_rect(clip);
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap_px;
                    ui.add_space(left_pad);

                    for (i, title) in tab_titles.iter().enumerate() {
                        let favicon = tab_favicons
                            .get(i)
                            .map(String::as_str)
                            .unwrap_or(crate::branding::TONET_LOGO_URI);
                        draw_tab(
                            ui,
                            i,
                            title,
                            favicon,
                            i == active_index,
                            can_close_any,
                            loc,
                            &mut out,
                            per_tab_w,
                        );
                    }

                    ui.add_space(theme::SP);

                    if ui
                        .add(
                            egui::Button::new(
                                RichText::new("+")
                                    .size(18.0)
                                    .color(theme::TAB_TEXT_MUTED),
                            )
                            .frame(false),
                        )
                        .on_hover_text(i18n::tab_new_tooltip(loc))
                        .clicked()
                    {
                        out.new_tab = true;
                    }

                    if integrated_caption {
                        let spare = ui.available_width();
                        if spare > 1.0 {
                            let drag = ui.allocate_response(
                                Vec2::new(spare, TAB_ROW_H),
                                Sense::click_and_drag(),
                            );
                            apply_drag_or_maximize(ctx, &drag);
                            drag.on_hover_text(i18n::window_drag_hint(loc));
                        }
                    }
                });
            },
        );

        if integrated_caption {
            let drag_gap = ui.allocate_response(
                Vec2::new(DRAG_GAP, TAB_ROW_H),
                Sense::click_and_drag(),
            );
            apply_drag_or_maximize(ctx, &drag_gap);
            drag_gap.on_hover_text(i18n::window_drag_hint(loc));

            show_window_caption_controls(ui, ctx, loc);
        }
    });

    out
}
