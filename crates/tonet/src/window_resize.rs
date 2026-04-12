//! Hit-test for native window resize when using undecorated (integrated) chrome.

use egui::{Context, Pos2, Rect, ResizeDirection, ViewportCommand};

/// Pixels from the window edge that count as a resize grip.
const EDGE: f32 = 6.0;
/// Ignore the top strip for east-edge resize so it does not fight caption buttons.
const EAST_TOP_SKIP: f32 = 44.0;

/// If the user pressed the primary button on a resize edge (and egui did not claim the click),
/// ask the compositor to start a native resize drag.
pub fn maybe_begin_native_resize(ctx: &Context, integrated: bool) {
    if !integrated {
        return;
    }
    if !ctx.input(|i| i.pointer.primary_pressed()) {
        return;
    }
    let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) else {
        return;
    };
    let screen = ctx.screen_rect();
    if let Some(dir) = hit_resize_direction(pos, screen) {
        ctx.send_viewport_cmd(ViewportCommand::BeginResize(dir));
    }
}

fn hit_resize_direction(pos: Pos2, s: Rect) -> Option<ResizeDirection> {
    let in_w = pos.x <= s.min.x + EDGE;
    let in_e = pos.x >= s.max.x - EDGE && pos.y >= s.min.y + EAST_TOP_SKIP;
    let in_s = pos.y >= s.max.y - EDGE;
    // Omit generic north edge: it fights the tab strip. Corners still allow diagonal resize.
    let in_n = pos.y <= s.min.y + EDGE;

    if in_s && in_w {
        return Some(ResizeDirection::SouthWest);
    }
    if in_s && in_e {
        return Some(ResizeDirection::SouthEast);
    }
    if in_s {
        return Some(ResizeDirection::South);
    }
    if in_e {
        return Some(ResizeDirection::East);
    }
    if in_w {
        return Some(ResizeDirection::West);
    }
    if in_n && in_w {
        return Some(ResizeDirection::NorthWest);
    }
    if in_n && in_e {
        return Some(ResizeDirection::NorthEast);
    }
    None
}
