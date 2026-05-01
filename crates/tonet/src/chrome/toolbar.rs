//! Browser toolbar row: < > ↻ | omnibox pill (responsive) | ☆ icons ☰
//!
//! No background frame -- the parent TopPanel provides `CHROME_BG`.

use std::hash::Hash;

use egui::text::{CCursor, CCursorRange};
use egui::{Align, Color32, Key, Layout, Modifiers, RichText, Shape, Stroke, Ui, Vec2};

use crate::browser_log::OmniboxHistorySuggestion;
use crate::i18n::{self, Locale};
use crate::theme;
use crate::ui::omnibox_id;

const TOOLBAR_H: f32 = 38.0;

fn omnibox_history_sel_id() -> egui::Id {
    omnibox_id().with("visit_history_kb_row")
}

fn omnibox_history_fp_id() -> egui::Id {
    omnibox_id().with("visit_history_kb_fp")
}

pub struct ToolbarResult {
    pub navigate: bool,
    pub reload: bool,
    pub stop_loading: bool,
    /// Open legacy modal settings (e.g. Ctrl/⌘+,). Hamburger uses `navigate_to_settings`.
    pub open_settings: bool,
    /// Navigate current tab to `tonet://settings` (main settings entry).
    pub navigate_to_settings: bool,
    pub go_back: bool,
    pub go_forward: bool,
}

fn chrome_button(
    ui: &mut Ui,
    id: impl Hash,
    glyph: &str,
    size: f32,
    color: Color32,
    enabled: bool,
) -> egui::Response {
    let hover_idx = ui.painter().add(Shape::Noop);
    let ir = ui.push_id(id, |ui| {
        ui.add_enabled(
            enabled,
            egui::Button::new(RichText::new(glyph).size(size).color(color))
                .min_size(Vec2::splat(theme::CHROME_BTN))
                .rounding(theme::CHROME_BTN_ROUNDING)
                .fill(Color32::TRANSPARENT),
        )
    });
    let resp = ir.inner;
    if resp.hovered() {
        ui.painter().set(
            hover_idx,
            Shape::rect_filled(resp.rect, theme::CHROME_BTN_ROUNDING, theme::tab_hover()),
        );
    }
    resp
}

#[allow(clippy::too_many_arguments)]
pub fn show_chrome_toolbar(
    ui: &mut Ui,
    loc: Locale,
    url_input: &mut String,
    chip_address_preview: &str,
    loading: bool,
    can_back: bool,
    can_forward: bool,
    focus_omnibox_select_all: bool,
    omnibox_visit_suggestions: &[OmniboxHistorySuggestion],
) -> ToolbarResult {
    let mut navigate = false;
    let mut reload = false;
    let mut stop_loading = false;
    let open_settings = false;
    let mut navigate_to_settings = false;
    let mut go_back = false;
    let mut go_forward = false;

    let nav_color = |enabled: bool| -> Color32 {
        if enabled {
            theme::nav_glyph()
        } else {
            theme::nav_glyph_disabled()
        }
    };

    ui.horizontal(|ui| {
        ui.set_height(TOOLBAR_H);
        ui.spacing_mut().item_spacing.x = theme::SP;
        ui.visuals_mut().widgets.hovered.bg_stroke = Stroke::NONE;
        ui.visuals_mut().widgets.active.bg_stroke = Stroke::NONE;
        ui.add_space(theme::SP);

        // ── Left: navigation buttons ────────────────────────────
        if chrome_button(
            ui,
            egui::Id::new("tonet_chrome_back"),
            "<",
            16.0,
            nav_color(can_back),
            can_back,
        )
        .on_hover_text(i18n::back_tooltip(loc))
        .clicked()
        {
            go_back = true;
        }

        if chrome_button(
            ui,
            egui::Id::new("tonet_chrome_forward"),
            ">",
            16.0,
            nav_color(can_forward),
            can_forward,
        )
        .on_hover_text(i18n::forward_tooltip(loc))
        .clicked()
        {
            go_forward = true;
        }

        if loading {
            if chrome_button(
                ui,
                egui::Id::new("tonet_chrome_stop"),
                "✕",
                14.0,
                theme::nav_glyph(),
                true,
            )
            .on_hover_text(i18n::stop_loading_tooltip(loc))
            .clicked()
            {
                stop_loading = true;
            }
        } else if chrome_button(
            ui,
            egui::Id::new("tonet_chrome_reload"),
            "↻",
            16.0,
            theme::nav_glyph(),
            true,
        )
        .on_hover_text(format!(
            "{}\n{}",
            i18n::reload_tooltip(loc),
            i18n::reload_shortcuts_hint(loc)
        ))
        .clicked()
        {
            reload = true;
        }

        ui.add_space(theme::SP2);

        // ── Reserve width for right icons (star + 6 icons + hamburger = 8) ──
        let right_count: f32 = 8.0;
        let right_block =
            right_count * theme::CHROME_BTN + (right_count - 1.0) * theme::SP + theme::SP3 + theme::SP;
        let omnibox_w = (ui.available_width() - right_block).max(200.0);

        // ── Center: omnibox pill ────────────────────────────────
        let (_chip_label, chip_tip) =
            i18n::security_chip_pair(chip_address_preview, loc);
        let chip_icon = if chip_address_preview.starts_with("https://") {
            "🔒"
        } else if chip_address_preview.starts_with("http://") {
            "⚠"
        } else {
            "🔍"
        };

        let mut url_enter = false;

        ui.vertical(|ui| {
            ui.set_width(omnibox_w);
            let mut omnibox_has_focus = false;
            ui.allocate_ui_with_layout(
                Vec2::new(omnibox_w, theme::CHROME_BTN),
                Layout::left_to_right(Align::Center),
                |ui| {
                    egui::Frame::none()
                        .fill(theme::omnibox_fill())
                        .stroke(Stroke::new(1.0, theme::omnibox_stroke()))
                        .rounding(20.0)
                        .inner_margin(egui::Margin::symmetric(theme::SP3, theme::SP + 2.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = theme::SP2;
                                ui.label(
                                    RichText::new(chip_icon)
                                        .size(14.0)
                                        .color(theme::chip()),
                                )
                                .on_hover_text(chip_tip);

                                let te = egui::TextEdit::singleline(url_input)
                                    .id(omnibox_id())
                                    .frame(false)
                                    .text_color(theme::omnibox_text())
                                    .hint_text(i18n::address_hint(loc))
                                    .desired_rows(1)
                                    .desired_width(ui.available_width());
                                let output = te.show(ui);
                                omnibox_has_focus = output.response.has_focus();

                                if focus_omnibox_select_all {
                                    let id = output.response.id;
                                    let n = url_input.chars().count();
                                    ui.ctx().memory_mut(|m| m.request_focus(id));
                                    let mut state = output.state;
                                    state.cursor.set_char_range(Some(
                                        CCursorRange::two(
                                            CCursor::new(0),
                                            CCursor::new(n),
                                        ),
                                    ));
                                    state.store(ui.ctx(), id);
                                }

                                output
                                    .response
                                    .on_hover_text(i18n::omnibox_focus_shortcut_hint(loc));
                            });
                        });
                },
            );

            let n_sugg = omnibox_visit_suggestions.len();
            let mut fp = String::with_capacity(url_input.len().saturating_add(n_sugg * 48));
            fp.push_str(url_input);
            for s in omnibox_visit_suggestions {
                fp.push('\n');
                fp.push_str(&s.url);
            }
            ui.ctx().data_mut(|d| {
                let stored = d.get_temp_mut_or_insert_with(omnibox_history_fp_id(), String::new);
                if *stored != fp {
                    *stored = fp;
                    *d.get_temp_mut_or_insert_with(omnibox_history_sel_id(), || None::<usize>) = None;
                }
            });

            let omnibox_focused = omnibox_has_focus || ui.ctx().memory(|m| m.has_focus(omnibox_id()));
            let mut kb_row: Option<usize> = ui
                .ctx()
                .data(|d| d.get_temp(omnibox_history_sel_id()))
                .flatten();

            if omnibox_focused && n_sugg > 0 && kb_row.is_some() && ui.input(|i| i.key_pressed(Key::Escape)) {
                kb_row = None;
                ui.ctx().data_mut(|d| {
                    *d.get_temp_mut_or_insert_with(omnibox_history_sel_id(), || None::<usize>) = None;
                });
                ui.ctx()
                    .input_mut(|i| i.consume_key(Modifiers::NONE, Key::Escape));
            }

            if omnibox_focused && n_sugg > 0 {
                if ui.input(|i| i.key_pressed(Key::ArrowDown)) {
                    kb_row = Some(kb_row.map_or(0, |i| (i + 1).min(n_sugg - 1)));
                }
                if ui.input(|i| i.key_pressed(Key::ArrowUp)) {
                    kb_row = match kb_row {
                        None => None,
                        Some(0) => None,
                        Some(i) => Some(i - 1),
                    };
                }
                ui.ctx().data_mut(|d| {
                    *d.get_temp_mut_or_insert_with(omnibox_history_sel_id(), || None::<usize>) = kb_row;
                });
            }

            if omnibox_focused && ui.input(|i| i.key_pressed(Key::Enter)) {
                if let Some(si) = kb_row {
                    if let Some(s) = omnibox_visit_suggestions.get(si) {
                        *url_input = s.url.clone();
                        navigate = true;
                        ui.ctx().data_mut(|d| {
                            *d.get_temp_mut_or_insert_with(omnibox_history_sel_id(), || None::<usize>) = None;
                        });
                        ui.ctx()
                            .input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
                    }
                } else {
                    url_enter = true;
                    ui.ctx()
                        .input_mut(|i| i.consume_key(Modifiers::NONE, Key::Enter));
                }
            }

            if omnibox_focused && !omnibox_visit_suggestions.is_empty() {
                ui.add_space(2.0);
                egui::Frame::none()
                    .fill(theme::omnibox_fill())
                    .stroke(Stroke::new(1.0, theme::omnibox_stroke()))
                    .rounding(10.0)
                    .inner_margin(egui::Margin::symmetric(theme::SP2, theme::SP))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(i18n::omnibox_history_heading(loc))
                                .small()
                                .color(theme::omnibox_text()),
                        )
                        .on_hover_text(i18n::omnibox_history_keyboard_hint(loc));
                        ui.add_space(2.0);
                        egui::ScrollArea::vertical()
                            .id_salt("tonet_omnibox_history_scroll")
                            .max_height(140.0)
                            .show(ui, |ui| {
                                for (idx, s) in omnibox_visit_suggestions.iter().enumerate() {
                                    let label = match &s.title {
                                        Some(t) if !t.trim().is_empty() => {
                                            format!("{}\n{}", s.url, t.trim())
                                        }
                                        _ => s.url.clone(),
                                    };
                                    let row_selected = kb_row == Some(idx);
                                    if ui
                                        .add(egui::SelectableLabel::new(
                                            row_selected,
                                            RichText::new(label).size(11.5),
                                        ))
                                        .clicked()
                                    {
                                        *url_input = s.url.clone();
                                        navigate = true;
                                        ui.ctx().data_mut(|d| {
                                            *d.get_temp_mut_or_insert_with(
                                                omnibox_history_sel_id(),
                                                || None::<usize>,
                                            ) = None;
                                        });
                                    }
                                }
                            });
                    });
            }
        });
        if url_enter {
            navigate = true;
        }

        ui.add_space(theme::SP);

        // ── Right: icon group ───────────────────────────────────
        ui.spacing_mut().item_spacing.x = theme::SP;

        let _ = chrome_button(ui, egui::Id::new("tonet_chrome_tool_1"), "☆", 16.0, theme::tool_icon(), true);
        let _ = chrome_button(ui, egui::Id::new("tonet_chrome_tool_2"), "◯", 16.0, theme::tool_icon(), true);
        let _ = chrome_button(ui, egui::Id::new("tonet_chrome_tool_3"), "⊛", 16.0, theme::tool_icon(), true);
        let _ = chrome_button(ui, egui::Id::new("tonet_chrome_tool_4"), "⤓", 16.0, theme::tool_icon(), true);
        let _ = chrome_button(ui, egui::Id::new("tonet_chrome_tool_5"), "↺", 16.0, theme::tool_icon(), true);
        let _ = chrome_button(ui, egui::Id::new("tonet_chrome_tool_6"), "⊞", 16.0, theme::tool_icon(), true);
        let _ = chrome_button(ui, egui::Id::new("tonet_chrome_tool_7"), "◎", 16.0, theme::tool_icon(), true);

        if chrome_button(
            ui,
            egui::Id::new("tonet_chrome_menu"),
            "☰",
            16.0,
            theme::tool_icon(),
            true,
        )
            .on_hover_text(i18n::settings_tooltip(loc))
            .clicked()
        {
            navigate_to_settings = true;
        }
    });

    ToolbarResult {
        navigate,
        reload,
        stop_loading,
        open_settings,
        navigate_to_settings,
        go_back,
        go_forward,
    }
}
