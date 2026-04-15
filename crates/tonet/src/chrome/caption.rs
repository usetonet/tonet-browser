//! Window caption controls (minimize, maximize/restore, close) for integrated title chrome.

use egui::{Align2, Color32, Context, FontId, RichText, Shape, Stroke, Ui, Vec2, ViewportCommand};

use crate::i18n::{self, Locale};
use crate::theme;

pub const CAPTION_BTN: Vec2 = Vec2::new(46.0, 36.0);

pub fn show_window_caption_controls(ui: &mut Ui, ctx: &Context, loc: Locale) {
    ui.spacing_mut().item_spacing.x = 0.0;
    ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::NONE;
    ui.visuals_mut().widgets.active.bg_stroke = Stroke::NONE;

    let cap_btn =
        |ui: &mut Ui, label: RichText, tip: &'static str, hover_color: Color32| -> bool {
            let hover_idx = ui.painter().add(Shape::Noop);
            let resp = ui.add(
                egui::Button::new(label)
                    .min_size(CAPTION_BTN)
                    .rounding(0.0)
                    .fill(Color32::TRANSPARENT),
            );
            if resp.hovered() {
                ui.painter().set(
                    hover_idx,
                    Shape::rect_filled(resp.rect, 0.0, hover_color),
                );
            }
            resp.on_hover_text(tip).clicked()
        };

    if cap_btn(
        ui,
        RichText::new("−").size(14.0).color(theme::caption_glyph()),
        i18n::window_minimize(loc),
        theme::tab_hover(),
    ) {
        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
    }

    let maximized = ctx.input(|i| i.viewport().maximized).unwrap_or(false);
    let (glyph, tip) = if maximized {
        ("❐", i18n::window_restore(loc))
    } else {
        ("□", i18n::window_maximize(loc))
    };
    if cap_btn(
        ui,
        RichText::new(glyph).size(12.0).color(theme::caption_glyph()),
        tip,
        theme::tab_hover(),
    ) {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    }

    {
        let close_hover_idx = ui.painter().add(Shape::Noop);
        let close = ui.add(
            egui::Button::new(RichText::new("").size(11.0))
                .min_size(CAPTION_BTN)
                .rounding(0.0)
                .fill(Color32::TRANSPARENT),
        );
        let glyph_color = if close.hovered() {
            ui.painter().set(
                close_hover_idx,
                Shape::rect_filled(close.rect, 0.0, theme::caption_close_hover()),
            );
            Color32::WHITE
        } else {
            theme::caption_close()
        };
        ui.painter().text(
            close.rect.center(),
            Align2::CENTER_CENTER,
            "✕",
            FontId::proportional(11.0),
            glyph_color,
        );
        if close.on_hover_text(i18n::window_close(loc)).clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
    }
}

pub fn apply_drag_or_maximize(ctx: &Context, resp: &egui::Response) {
    if resp.drag_started() {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if resp.double_clicked() {
        let maximized = ctx.input(|i| i.viewport().maximized).unwrap_or(false);
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    }
}
