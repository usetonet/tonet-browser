//! Browser-style developer tools shell (Chrome / Firefox layout).
//!
//! Console is wired to Servo embedder output; other panels are placeholders until
//! inspector, network, and debugger APIs land.

mod elements;
mod network;
mod scrollbar;

use egui::{self, Color32, Rect, RichText, Sense, Ui, Vec2};

use crate::i18n::{self, Locale};
use crate::servo_engine::embedder_devtools::{ServoDomTreeNode, ServoNetworkEntry};
use crate::tab::ServoConsoleLevel;
use crate::theme;

pub use scrollbar::{paint_servo_scrollbar, ServoScrollbarParams};

/// Where the DevTools pane is anchored relative to the page.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DevToolsDock {
    #[default]
    Right,
    Bottom,
}

/// Primary DevTools tabs (top bar, Chrome-style order).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DevToolsPanel {
    #[default]
    Console,
    Elements,
    Sources,
    Network,
    Performance,
    Memory,
    Application,
    Security,
}

/// Secondary drawer tabs at the bottom of the DevTools pane.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DevToolsDrawerPanel {
    #[default]
    Console,
    NetworkConditions,
    WhatsNew,
    Issues,
}

pub struct DevToolsResponse {
    pub close: bool,
}

const SPLIT_MIN_PAGE: f32 = 160.0;
const SPLIT_MIN_DEVTOOLS: f32 = 220.0;
const SPLIT_HANDLE: f32 = 5.0;
const DRAWER_H: f32 = 132.0;
const TOOLBAR_H: f32 = 28.0;
const TAB_BAR_H: f32 = 26.0;
const DRAWER_TAB_H: f32 = 24.0;

pub fn default_split_ratio(dock: DevToolsDock) -> f32 {
    match dock {
        DevToolsDock::Right => 0.46,
        DevToolsDock::Bottom => 0.38,
    }
}

/// Split the central content area between the page and DevTools.
pub fn split_page_and_devtools(
    full: Rect,
    open: bool,
    dock: DevToolsDock,
    split: f32,
) -> (Rect, Option<Rect>) {
    if !open {
        return (full, None);
    }
    let split = split.clamp(0.18, 0.72);
    match dock {
        DevToolsDock::Right => {
            let dt_w = (full.width() * split)
                .clamp(SPLIT_MIN_DEVTOOLS, full.width() - SPLIT_MIN_PAGE);
            let page_max_x = full.max.x - dt_w;
            let page = Rect::from_min_max(full.min, egui::pos2(page_max_x, full.max.y));
            let dt = Rect::from_min_max(egui::pos2(page_max_x, full.min.y), full.max);
            (page, Some(dt))
        }
        DevToolsDock::Bottom => {
            let dt_h = (full.height() * split)
                .clamp(SPLIT_MIN_DEVTOOLS, full.height() - SPLIT_MIN_PAGE);
            let page_max_y = full.max.y - dt_h;
            let page = Rect::from_min_max(full.min, egui::pos2(full.max.x, page_max_y));
            let dt = Rect::from_min_max(egui::pos2(full.min.x, page_max_y), full.max);
            (page, Some(dt))
        }
    }
}

/// Draggable split between page and DevTools; call after laying out both rects.
pub fn paint_split_handle(
    ui: &mut Ui,
    page_rect: Rect,
    devtools_rect: Rect,
    dock: DevToolsDock,
    split: &mut f32,
) {
    let handle_rect = match dock {
        DevToolsDock::Right => Rect::from_min_max(
            egui::pos2(page_rect.max.x - SPLIT_HANDLE * 0.5, page_rect.min.y),
            egui::pos2(page_rect.max.x + SPLIT_HANDLE * 0.5, page_rect.max.y),
        ),
        DevToolsDock::Bottom => Rect::from_min_max(
            egui::pos2(page_rect.min.x, page_rect.max.y - SPLIT_HANDLE * 0.5),
            egui::pos2(page_rect.max.x, page_rect.max.y + SPLIT_HANDLE * 0.5),
        ),
    };
    let resp = ui.allocate_rect(handle_rect, Sense::drag());
    if resp.dragged() {
        match dock {
            DevToolsDock::Right => {
                let parent_w = page_rect.width() + devtools_rect.width();
                if parent_w > 1.0 {
                    let new_dt = (devtools_rect.width() - resp.drag_delta().x)
                        .clamp(SPLIT_MIN_DEVTOOLS, parent_w - SPLIT_MIN_PAGE);
                    *split = (new_dt / parent_w).clamp(0.18, 0.72);
                }
            }
            DevToolsDock::Bottom => {
                let parent_h = page_rect.height() + devtools_rect.height();
                if parent_h > 1.0 {
                    let new_dt = (devtools_rect.height() - resp.drag_delta().y)
                        .clamp(SPLIT_MIN_DEVTOOLS, parent_h - SPLIT_MIN_PAGE);
                    *split = (new_dt / parent_h).clamp(0.18, 0.72);
                }
            }
        }
    }
    if resp.hovered() || resp.dragged() {
        let icon = match dock {
            DevToolsDock::Right => egui::CursorIcon::ResizeHorizontal,
            DevToolsDock::Bottom => egui::CursorIcon::ResizeVertical,
        };
        ui.ctx().set_cursor_icon(icon);
    }
    ui.painter().rect_filled(handle_rect, 0.0, theme::devtools_split_handle());
}

#[allow(clippy::too_many_arguments)]
pub fn show_devtools(
    ui: &mut Ui,
    loc: Locale,
    rect: Rect,
    dock: &mut DevToolsDock,
    split: &mut f32,
    active_panel: &mut DevToolsPanel,
    drawer_open: &mut bool,
    drawer_panel: &mut DevToolsDrawerPanel,
    servo_console: &[(ServoConsoleLevel, String)],
    network_log: &[ServoNetworkEntry],
    dom_root: Option<&ServoDomTreeNode>,
    dom_error: Option<&str>,
    dom_loading: bool,
    on_clear_console: &mut dyn FnMut(),
    on_clear_network: &mut dyn FnMut(),
    on_refresh_dom: &mut dyn FnMut(),
) -> DevToolsResponse {
    let mut close = false;
    ui.allocate_new_ui(
        egui::UiBuilder::new()
            .id_salt(egui::Id::new("tonet_devtools_root"))
            .max_rect(rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
        |ui| {
            ui.set_min_size(rect.size());
            egui::Frame::default()
                .fill(theme::devtools_bg())
                .stroke(egui::Stroke::new(1.0, theme::separator()))
                .show(ui, |ui| {
                    show_devtools_toolbar(
                        ui,
                        loc,
                        dock,
                        split,
                        drawer_open,
                        &mut close,
                    );
                    ui.add_space(1.0);
                    show_main_tab_bar(ui, loc, active_panel);
                    let drawer_h = if *drawer_open {
                        DRAWER_H + DRAWER_TAB_H
                    } else {
                        0.0
                    };
                    let main_h = (ui.available_height() - drawer_h).max(40.0);
                    ui.allocate_ui_with_layout(
                        Vec2::new(ui.available_width(), main_h),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            show_panel_body(
                                ui,
                                loc,
                                *active_panel,
                                servo_console,
                                network_log,
                                dom_root,
                                dom_error,
                                dom_loading,
                                on_clear_console,
                                on_clear_network,
                                on_refresh_dom,
                            );
                        },
                    );
                    if *drawer_open {
                        show_drawer(ui, loc, drawer_panel, servo_console, on_clear_console);
                    }
                });
        },
    );
    DevToolsResponse { close }
}

fn show_devtools_toolbar(
    ui: &mut Ui,
    loc: Locale,
    dock: &mut DevToolsDock,
    split: &mut f32,
    drawer_open: &mut bool,
    close: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.set_height(TOOLBAR_H);
        ui.spacing_mut().item_spacing.x = theme::SP;
        ui.add_space(theme::SP);
        toolbar_icon(ui, "⬚", i18n::devtools_inspect_tooltip(loc), false);
        toolbar_icon(ui, "📱", i18n::devtools_device_tooltip(loc), false);
        ui.add_space(theme::SP2);
        ui.menu_button(
            RichText::new("⋮").size(16.0).color(theme::devtools_toolbar_icon()),
            |ui| {
                if ui
                    .selectable_label(*dock == DevToolsDock::Right, i18n::devtools_dock_right(loc))
                    .clicked()
                {
                    *dock = DevToolsDock::Right;
                    ui.close_menu();
                }
                if ui
                    .selectable_label(
                        *dock == DevToolsDock::Bottom,
                        i18n::devtools_dock_bottom(loc),
                    )
                    .clicked()
                {
                    *dock = DevToolsDock::Bottom;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button(i18n::devtools_toggle_drawer(loc)).clicked() {
                    *drawer_open = !*drawer_open;
                    ui.close_menu();
                }
            },
        )
        .response
        .on_hover_text(i18n::devtools_menu_tooltip(loc));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if toolbar_icon(ui, "✕", i18n::devtools_close_tooltip(loc), true) {
                *close = true;
            }
            toolbar_icon(ui, "⚙", i18n::devtools_settings_tooltip(loc), false);
            let dock_glyph = match *dock {
                DevToolsDock::Right => "◧",
                DevToolsDock::Bottom => "◨",
            };
            if ui
                .add(
                    egui::Button::new(
                        RichText::new(dock_glyph)
                            .size(14.0)
                            .color(theme::devtools_toolbar_icon()),
                    )
                    .frame(false),
                )
                .on_hover_text(i18n::devtools_cycle_dock_tooltip(loc))
                .clicked()
            {
                *dock = match *dock {
                    DevToolsDock::Right => DevToolsDock::Bottom,
                    DevToolsDock::Bottom => DevToolsDock::Right,
                };
                *split = default_split_ratio(*dock);
            }
        });
    });
}

fn toolbar_icon(ui: &mut Ui, glyph: &str, tip: &str, enabled: bool) -> bool {
    ui.add_enabled(
        enabled,
        egui::Button::new(RichText::new(glyph).size(14.0).color(theme::devtools_toolbar_icon()))
            .frame(false),
    )
    .on_hover_text(tip)
    .clicked()
}

fn show_main_tab_bar(ui: &mut Ui, loc: Locale, active: &mut DevToolsPanel) {
    ui.horizontal(|ui| {
        ui.set_height(TAB_BAR_H);
        ui.spacing_mut().item_spacing.x = 0.0;
        for panel in [
            DevToolsPanel::Console,
            DevToolsPanel::Elements,
            DevToolsPanel::Sources,
            DevToolsPanel::Network,
            DevToolsPanel::Performance,
            DevToolsPanel::Memory,
            DevToolsPanel::Application,
            DevToolsPanel::Security,
        ] {
            let selected = *active == panel;
            let label = panel_label(loc, panel);
            let fill = if selected {
                theme::devtools_tab_active()
            } else {
                Color32::TRANSPARENT
            };
            let text = if selected {
                theme::devtools_tab_text_active()
            } else {
                theme::devtools_tab_text_idle()
            };
            let r = ui.add(
                egui::Button::new(RichText::new(label).size(12.5).color(text))
                    .fill(fill)
                    .stroke(egui::Stroke::NONE)
                    .rounding(0.0)
                    .min_size(Vec2::new(0.0, TAB_BAR_H)),
            );
            if r.clicked() {
                *active = panel;
            }
        }
    });
    ui.painter().hline(
        ui.max_rect().x_range(),
        ui.max_rect().top(),
        egui::Stroke::new(1.0, theme::separator()),
    );
}

fn show_drawer(
    ui: &mut Ui,
    loc: Locale,
    drawer_panel: &mut DevToolsDrawerPanel,
    servo_console: &[(ServoConsoleLevel, String)],
    on_clear_console: &mut dyn FnMut(),
) {
    ui.painter().hline(
        ui.max_rect().x_range(),
        ui.max_rect().top(),
        egui::Stroke::new(1.0, theme::separator()),
    );
    ui.horizontal(|ui| {
        ui.set_height(DRAWER_TAB_H);
        for p in [
            DevToolsDrawerPanel::Console,
            DevToolsDrawerPanel::NetworkConditions,
            DevToolsDrawerPanel::WhatsNew,
            DevToolsDrawerPanel::Issues,
        ] {
            let selected = *drawer_panel == p;
            let text = drawer_panel_label(loc, p);
            let color = if selected {
                theme::devtools_tab_text_active()
            } else {
                theme::devtools_tab_text_idle()
            };
            if ui
                .add(
                    egui::Button::new(RichText::new(text).size(11.5).color(color))
                        .frame(false),
                )
                .clicked()
            {
                *drawer_panel = p;
            }
        }
    });
    ui.allocate_ui_with_layout(
        Vec2::new(ui.available_width(), DRAWER_H),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            match drawer_panel {
                DevToolsDrawerPanel::Console => {
                    show_console_body(ui, loc, servo_console, on_clear_console);
                }
                DevToolsDrawerPanel::NetworkConditions
                | DevToolsDrawerPanel::WhatsNew
                | DevToolsDrawerPanel::Issues => {
                    ui.add_space(theme::SP2);
                    ui.label(
                        RichText::new(i18n::devtools_panel_coming_soon(loc))
                            .small()
                            .color(theme::loading_muted()),
                    );
                }
            }
        },
    );
}

fn show_panel_body(
    ui: &mut Ui,
    loc: Locale,
    panel: DevToolsPanel,
    servo_console: &[(ServoConsoleLevel, String)],
    network_log: &[ServoNetworkEntry],
    dom_root: Option<&ServoDomTreeNode>,
    dom_error: Option<&str>,
    dom_loading: bool,
    on_clear_console: &mut dyn FnMut(),
    on_clear_network: &mut dyn FnMut(),
    on_refresh_dom: &mut dyn FnMut(),
) {
    match panel {
        DevToolsPanel::Console => show_console_body(ui, loc, servo_console, on_clear_console),
        DevToolsPanel::Elements => elements::show_elements_panel(
            ui,
            loc,
            dom_root,
            dom_error,
            dom_loading,
            on_refresh_dom,
        ),
        DevToolsPanel::Network => {
            network::show_network_panel(ui, loc, network_log, on_clear_network);
        }
        _ => {
            ui.add_space(theme::SP3);
            ui.label(
                RichText::new(panel_heading(loc, panel))
                    .strong()
                    .color(theme::devtools_tab_text_active()),
            );
            ui.add_space(theme::SP2);
            ui.label(
                RichText::new(i18n::devtools_panel_coming_soon(loc))
                    .color(theme::loading_muted()),
            );
            ui.add_space(theme::SP);
            ui.label(
                RichText::new(i18n::devtools_panel_coming_soon_detail(loc))
                    .small()
                    .color(theme::loading_muted()),
            );
        }
    }
}

fn show_console_body(
    ui: &mut Ui,
    loc: Locale,
    servo_console: &[(ServoConsoleLevel, String)],
    on_clear_console: &mut dyn FnMut(),
) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(i18n::devtools_console_filter(loc))
                .small()
                .color(theme::loading_muted()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(i18n::servo_page_console_clear(loc)).clicked() {
                on_clear_console();
            }
        });
    });
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.style_mut().override_font_id = Some(egui::FontId::monospace(11.5));
            if servo_console.is_empty() {
                ui.label(
                    RichText::new(i18n::devtools_console_empty(loc))
                        .small()
                        .color(theme::loading_muted()),
                );
            } else {
                for (lvl, text) in servo_console {
                    let c = console_line_color(*lvl);
                    let line = format!("[{}] {}", lvl.as_label(), text);
                    ui.label(RichText::new(line).small().color(c));
                }
            }
        });
}

fn console_line_color(level: ServoConsoleLevel) -> Color32 {
    use ServoConsoleLevel as L;
    match level {
        L::Error => theme::error_title(),
        L::Warn => Color32::from_rgb(230, 178, 92),
        L::Info | L::Log => theme::accent(),
        L::Debug | L::Trace => theme::loading_muted(),
    }
}

fn panel_label(loc: Locale, panel: DevToolsPanel) -> &'static str {
    match panel {
        DevToolsPanel::Console => i18n::devtools_tab_console(loc),
        DevToolsPanel::Elements => i18n::devtools_tab_elements(loc),
        DevToolsPanel::Sources => i18n::devtools_tab_sources(loc),
        DevToolsPanel::Network => i18n::devtools_tab_network(loc),
        DevToolsPanel::Performance => i18n::devtools_tab_performance(loc),
        DevToolsPanel::Memory => i18n::devtools_tab_memory(loc),
        DevToolsPanel::Application => i18n::devtools_tab_application(loc),
        DevToolsPanel::Security => i18n::devtools_tab_security(loc),
    }
}

fn panel_heading(loc: Locale, panel: DevToolsPanel) -> &'static str {
    panel_label(loc, panel)
}

fn drawer_panel_label(loc: Locale, panel: DevToolsDrawerPanel) -> &'static str {
    match panel {
        DevToolsDrawerPanel::Console => i18n::devtools_tab_console(loc),
        DevToolsDrawerPanel::NetworkConditions => i18n::devtools_drawer_network_conditions(loc),
        DevToolsDrawerPanel::WhatsNew => i18n::devtools_drawer_whats_new(loc),
        DevToolsDrawerPanel::Issues => i18n::devtools_drawer_issues(loc),
    }
}
