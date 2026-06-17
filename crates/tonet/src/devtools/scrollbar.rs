//! Decorative Servo viewport scrollbar (shell-side; no scroll offset API yet).

use egui::{self, Rect, Sense, Ui};

use crate::servo_engine::ServoViewportRuntime;
use crate::theme;

pub struct ServoScrollbarParams<'a> {
    pub viewport: &'a ServoViewportRuntime,
    pub inner: Rect,
    pub full_max_x: f32,
    pub ppp: f32,
}

pub fn paint_servo_scrollbar(ui: &mut Ui, params: ServoScrollbarParams<'_>) {
    let ServoScrollbarParams {
        viewport,
        inner,
        full_max_x,
        ppp,
    } = params;

    let track = egui::Rect::from_min_max(
        egui::pos2(inner.right(), inner.top()),
        egui::pos2(full_max_x, inner.max.y),
    );
    let arrow_h = theme::SERVO_SCROLL_ARROW_H;
    let scroll_track = egui::Rect::from_min_max(
        egui::pos2(track.left(), track.top() + arrow_h),
        egui::pos2(track.right(), track.bottom() - arrow_h),
    );
    let thumb_h = (scroll_track.height() * 0.18).clamp(32.0, scroll_track.height());
    let travel = (scroll_track.height() - thumb_h).max(0.0);
    let thumb_top = scroll_track.top() + viewport.scroll_thumb_ratio() * travel;
    let thumb_rect = egui::Rect::from_min_size(
        egui::pos2(track.left() + 2.0, thumb_top),
        egui::vec2(track.width() - 4.0, thumb_h),
    );
    let thumb_resp = ui.allocate_rect(thumb_rect, Sense::click_and_drag());
    if thumb_resp.dragged() {
        viewport.scroll_by_pixels(
            0.0,
            thumb_resp.drag_delta().y,
            inner,
            ppp,
            thumb_resp.hover_pos(),
        );
    }
    let track_resp = ui.allocate_rect(scroll_track, Sense::click());
    if track_resp.clicked() {
        if let Some(pos) = track_resp.interact_pointer_pos() {
            let old_ratio = viewport.scroll_thumb_ratio();
            let ratio =
                ((pos.y - scroll_track.top()) / scroll_track.height()).clamp(0.0, 1.0);
            viewport.set_scroll_thumb_ratio(ratio);
            let jump = (ratio - old_ratio) * scroll_track.height();
            viewport.scroll_by_pixels(0.0, jump, inner, ppp, Some(pos));
        }
    }
    let arrow_up =
        egui::Rect::from_min_size(track.min, egui::vec2(track.width(), arrow_h));
    let arrow_down = egui::Rect::from_min_size(
        egui::pos2(track.left(), track.bottom() - arrow_h),
        egui::vec2(track.width(), arrow_h),
    );
    let up_resp = ui.allocate_rect(arrow_up, Sense::click());
    if up_resp.clicked() {
        viewport.scroll_by_pixels(0.0, -120.0, inner, ppp, up_resp.hover_pos());
    }
    let down_resp = ui.allocate_rect(arrow_down, Sense::click());
    if down_resp.clicked() {
        viewport.scroll_by_pixels(0.0, 120.0, inner, ppp, down_resp.hover_pos());
    }
    let p = ui.painter();
    p.rect_filled(track, 2.0, theme::servo_scroll_gutter_fill());
    let thumb_round = thumb_rect.width() * 0.5;
    p.rect_filled(thumb_rect, thumb_round, theme::servo_scroll_thumb());
    let arrow_c = theme::servo_scroll_arrow();
    let tri = |center: egui::Pos2, up: bool| {
        let h = arrow_h * 0.28;
        let w = track.width() * 0.22;
        let y = if up { h } else { -h };
        egui::Shape::convex_polygon(
            vec![
                center + egui::vec2(0.0, -y),
                center + egui::vec2(-w, y),
                center + egui::vec2(w, y),
            ],
            arrow_c,
            egui::Stroke::NONE,
        )
    };
    p.add(tri(arrow_up.center(), true));
    p.add(tri(arrow_down.center(), false));
}
