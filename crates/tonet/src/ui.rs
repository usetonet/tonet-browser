//! Browser chrome (toolbar, omnibox, settings) — layout inspired by mainstream browsers.

use egui::text::{CCursor, CCursorRange};
use egui::{
    Align, Color32, Context, Id, Layout, RichText, Sense, Ui, Vec2, ViewportCommand,
};

use crate::i18n::Locale;
use crate::i18n;
use crate::settings::{AppSettings, UpdatePolicy};

/// Stable [`Id`] for the omnibox so shortcuts can request focus and selection.
#[inline]
pub fn omnibox_id() -> Id {
    Id::new("tonet_omnibox")
}

/// Result of the main toolbar (omnibox row).
pub struct ToolbarResult {
    pub navigate: bool,
    pub reload: bool,
    pub stop_loading: bool,
    pub open_settings: bool,
    pub go_back: bool,
    pub go_forward: bool,
}

/// Tab strip: switch tab, open new, close.
#[derive(Default)]
pub struct TabBarResult {
    pub new_tab: bool,
    pub select_tab: Option<usize>,
    pub close_tab: Option<usize>,
}

/// Compact caption buttons (integrated title chrome).
const CAPTION_BTN: Vec2 = Vec2::new(32.0, 24.0);

fn show_window_caption_controls(ui: &mut Ui, ctx: &Context, loc: Locale) {
    ui.spacing_mut().item_spacing.x = 3.0;

    let cap_btn = |ui: &mut Ui, label: RichText, tip: &'static str| -> bool {
        ui.add(
            egui::Button::new(label)
                .min_size(CAPTION_BTN)
                .rounding(5.0)
                .fill(Color32::from_rgb(40, 42, 48)),
        )
        .on_hover_text(tip)
        .clicked()
    };

    if cap_btn(
        ui,
        RichText::new("−").size(15.0).color(Color32::from_gray(230)),
        i18n::window_minimize(loc),
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
        RichText::new(glyph).size(12.0).color(Color32::from_gray(225)),
        tip,
    ) {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    }

    let close = ui
        .add(
            egui::Button::new(RichText::new("✕").size(11.0).color(Color32::from_rgb(222, 118, 122)))
                .min_size(CAPTION_BTN)
                .rounding(5.0)
                .fill(Color32::from_rgb(40, 42, 48)),
        )
        .on_hover_text(i18n::window_close(loc));
    if close.clicked() {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }
}

fn apply_drag_or_maximize(ctx: &Context, resp: &egui::Response) {
    if resp.drag_started() {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if resp.double_clicked() {
        let maximized = ctx.input(|i| i.viewport().maximized).unwrap_or(false);
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    }
}

/// Horizontal tab strip (familiar browser layout) above the navigation toolbar.
pub fn show_tab_bar(
    ui: &mut Ui,
    ctx: &Context,
    loc: Locale,
    tab_titles: &[String],
    active_index: usize,
    can_close_any: bool,
    integrated_caption: bool,
) -> TabBarResult {
    let mut out = TabBarResult::default();
    let strip_bg = Color32::from_rgb(26, 28, 34);
    let row_h = 30.0;
    // Caption column + dedicated drag gap + inner padding on the right (avoids clipped ✕).
    const DRAG_GAP: f32 = 28.0;
    let caption_block = CAPTION_BTN.x * 3.0 + 3.0 * 2.0 + 6.0;
    let right_chrome = if integrated_caption {
        DRAG_GAP + caption_block + 6.0
    } else {
        0.0
    };

    let inner = if integrated_caption {
        egui::Margin {
            left: 8.0,
            right: 12.0,
            top: 5.0,
            bottom: 5.0,
        }
    } else {
        egui::Margin::symmetric(6.0, 5.0)
    };

    egui::Frame::default()
        .fill(strip_bg)
        .stroke(egui::Stroke::new(
            1.0,
            Color32::from_rgb(38, 40, 48),
        ))
        .inner_margin(inner)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let scroll_w = (ui.available_width() - right_chrome).max(64.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(scroll_w, row_h),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        egui::ScrollArea::horizontal()
                            .id_salt("tonet_tab_scroll")
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.spacing_mut().item_spacing.x = 4.0;
                                    for (i, title) in tab_titles.iter().enumerate() {
                                        let selected = i == active_index;
                                        let tab_bg = if selected {
                                            Color32::from_rgb(44, 46, 54)
                                        } else {
                                            Color32::from_rgb(34, 36, 42)
                                        };
                                        let rounding = egui::Rounding {
                                            nw: 6.0,
                                            ne: 6.0,
                                            sw: 0.0,
                                            se: 0.0,
                                        };
                                        ui.push_id(i as i32, |ui| {
                                            egui::Frame::default()
                                                .fill(tab_bg)
                                                .stroke(if selected {
                                                    egui::Stroke::new(
                                                        1.0,
                                                        Color32::from_rgb(88, 130, 220),
                                                    )
                                                } else {
                                                    egui::Stroke::NONE
                                                })
                                                .inner_margin(egui::Margin::symmetric(10.0, 5.0))
                                                .rounding(rounding)
                                                .show(ui, |ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.spacing_mut().item_spacing.x = 4.0;
                                                        let label = RichText::new(title.as_str())
                                                            .small()
                                                            .color(if selected {
                                                                Color32::from_gray(245)
                                                            } else {
                                                                Color32::from_gray(195)
                                                            });
                                                        if ui
                                                            .add(egui::SelectableLabel::new(
                                                                selected, label,
                                                            ))
                                                            .clicked()
                                                        {
                                                            out.select_tab = Some(i);
                                                        }
                                                        if can_close_any {
                                                            let close = ui
                                                                .add_sized(
                                                                    Vec2::new(22.0, 22.0),
                                                                    egui::Button::new(
                                                                        RichText::new("×")
                                                                            .size(15.0),
                                                                    ),
                                                                )
                                                                .on_hover_text(
                                                                    i18n::tab_close_tooltip(loc),
                                                                );
                                                            if close.clicked() {
                                                                out.close_tab = Some(i);
                                                            }
                                                        }
                                                    });
                                                });
                                        });
                                    }
                                    if ui
                                        .add_sized(
                                            Vec2::new(30.0, 28.0),
                                            egui::Button::new(
                                                RichText::new("+").strong().size(15.0),
                                            ),
                                        )
                                        .on_hover_text(i18n::tab_new_tooltip(loc))
                                        .clicked()
                                    {
                                        out.new_tab = true;
                                    }

                                    // Empty strip to the right of "+" = drag surface (entire top row).
                                    if integrated_caption {
                                        let spare = ui.available_width();
                                        if spare > 1.0 {
                                            let drag = ui.allocate_response(
                                                Vec2::new(spare, row_h),
                                                Sense::click_and_drag(),
                                            );
                                            apply_drag_or_maximize(ctx, &drag);
                                            drag.on_hover_text(i18n::window_drag_hint(loc));
                                        }
                                    }
                                });
                            });
                    },
                );

                if integrated_caption {
                    let drag_gap = ui.allocate_response(
                        Vec2::new(DRAG_GAP, row_h),
                        Sense::click_and_drag(),
                    );
                    apply_drag_or_maximize(ctx, &drag_gap);
                    drag_gap.on_hover_text(i18n::window_drag_hint(loc));

                    show_window_caption_controls(ui, ctx, loc);
                }
            });
        });

    out
}

/// Chromium-style row: back / forward / reload, security chip, URL, Go, settings.
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
) -> ToolbarResult {
    let mut navigate = false;
    let mut reload = false;
    let mut stop_loading = false;
    let mut open_settings = false;
    let mut go_back = false;
    let mut go_forward = false;

    let bar_bg = Color32::from_rgb(32, 34, 40);
    let btn_size = Vec2::new(34.0, 30.0);

    egui::Frame::default()
        .fill(bar_bg)
        .inner_margin(egui::Margin::symmetric(6.0, 4.0))
        .rounding(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let b_back = ui
                    .add_enabled(can_back, egui::Button::new(RichText::new("←").size(18.0)).min_size(btn_size))
                    .on_hover_text(i18n::back_tooltip(loc));
                if b_back.clicked() {
                    go_back = true;
                }

                let b_fwd = ui
                    .add_enabled(
                        can_forward,
                        egui::Button::new(RichText::new("→").size(18.0)).min_size(btn_size),
                    )
                    .on_hover_text(i18n::forward_tooltip(loc));
                if b_fwd.clicked() {
                    go_forward = true;
                }

                if loading {
                    let b_stop = ui
                        .add(egui::Button::new(RichText::new("⏹").size(14.0)).min_size(btn_size))
                        .on_hover_text(i18n::stop_loading_tooltip(loc));
                    if b_stop.clicked() {
                        stop_loading = true;
                    }
                } else {
                    let b_reload = ui
                        .add(egui::Button::new(RichText::new("↻").size(18.0)).min_size(btn_size))
                        .on_hover_text(format!(
                            "{}\n{}",
                            i18n::reload_tooltip(loc),
                            i18n::reload_shortcuts_hint(loc)
                        ));
                    if b_reload.clicked() {
                        reload = true;
                    }
                }

                ui.separator();

                let (chip_label, chip_tip) = i18n::security_chip_pair(chip_address_preview, loc);
                let chip_icon = if chip_label.starts_with("HTTPS") {
                    "🔒"
                } else if chip_label.starts_with("HTTP ·") {
                    "⚠"
                } else {
                    "◌"
                };
                ui.add(
                    egui::Label::new(
                        RichText::new(format!("{chip_icon}  {chip_label}"))
                            .small()
                            .color(Color32::from_gray(180)),
                    )
                    .truncate(),
                )
                .on_hover_text(chip_tip);

                let url_w = (ui.available_width() - 130.0).max(80.0);
                let mut url_enter = false;
                ui.allocate_ui_with_layout(
                    Vec2::new(url_w, 28.0),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        let output = egui::TextEdit::singleline(url_input)
                            .id(omnibox_id())
                            .hint_text(i18n::address_hint(loc))
                            .desired_rows(1)
                            .show(ui);

                        if focus_omnibox_select_all {
                            let id = output.response.id;
                            let n = url_input.chars().count();
                            ui.ctx().memory_mut(|m| m.request_focus(id));
                            let mut state = output.state;
                            state.cursor.set_char_range(Some(CCursorRange::two(
                                CCursor::new(0),
                                CCursor::new(n),
                            )));
                            state.store(ui.ctx(), id);
                        }

                        if output.response.has_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                        {
                            url_enter = true;
                        }
                        output
                            .response
                            .on_hover_text(i18n::omnibox_focus_shortcut_hint(loc));
                    },
                );
                if url_enter {
                    navigate = true;
                }

                let go_label = if loading {
                    i18n::go_loading(loc)
                } else {
                    i18n::go(loc)
                };
                let go = ui.add_sized(
                    Vec2::new(56.0, 30.0),
                    egui::Button::new(RichText::new(go_label).strong()),
                );
                if go.clicked() {
                    navigate = true;
                }

                let settings_btn = ui
                    .add_sized(Vec2::new(38.0, 30.0), egui::Button::new(RichText::new("⚙").size(16.0)))
                    .on_hover_text(i18n::settings_tooltip(loc));
                if settings_btn.clicked() {
                    open_settings = true;
                }
            });
        });

    ToolbarResult {
        navigate,
        reload,
        stop_loading,
        open_settings,
        go_back,
        go_forward,
    }
}

pub fn show_error_panel(ui: &mut Ui, loc: Locale, message: &str) {
    egui::Frame::default()
        .fill(Color32::from_rgb(72, 28, 28))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(140, 60, 60)))
        .inner_margin(14.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.with_layout(Layout::top_down(Align::Min), |ui| {
                ui.label(
                    RichText::new(i18n::error_title(loc))
                        .strong()
                        .color(Color32::from_rgb(255, 160, 160)),
                );
                ui.add_space(6.0);
                ui.label(RichText::new(message).color(Color32::from_rgb(255, 220, 220)));
            });
        });
}

pub fn show_loading(ui: &mut Ui, loc: Locale) {
    ui.vertical_centered(|ui| {
        ui.add_space(32.0);
        ui.spinner();
        ui.add_space(10.0);
        ui.label(RichText::new(i18n::loading_title(loc)).size(18.0).strong());
        ui.add_space(6.0);
        ui.label(
            RichText::new(i18n::loading_sub(loc))
                .small()
                .color(Color32::GRAY),
        );
    });
}

pub fn show_update_banner(
    ui: &mut Ui,
    loc: Locale,
    version_label: &str,
    on_open_downloads: impl FnOnce(),
    on_dismiss: impl FnOnce(),
) {
    egui::Frame::default()
        .fill(Color32::from_rgb(28, 52, 88))
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(80, 120, 200)))
        .inner_margin(12.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(i18n::update_banner_title(loc))
                        .strong()
                        .color(Color32::WHITE),
                );
                ui.label(
                    RichText::new(version_label)
                        .strong()
                        .color(Color32::from_rgb(180, 210, 255)),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(i18n::update_dismiss(loc)).clicked() {
                        on_dismiss();
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new(i18n::update_download(loc)).strong())
                                .fill(Color32::from_rgb(70, 130, 220)),
                        )
                        .clicked()
                    {
                        on_open_downloads();
                    }
                });
            });
        });
}

#[allow(clippy::too_many_arguments)]
pub fn show_settings_window(
    ctx: &egui::Context,
    open: &mut bool,
    settings: &mut AppSettings,
    loc: Locale,
    update_busy: bool,
    status_line: &str,
    current_version: &str,
    mut on_save: impl FnMut(&AppSettings),
    mut on_check_now: impl FnMut(),
) {
    let win = egui::Window::new(i18n::settings_window_title(loc))
        .open(open)
        .collapsible(false)
        .resizable(true)
        .default_width(460.0)
        .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
        .frame(
            egui::Frame::window(&ctx.style())
                .fill(Color32::from_rgb(36, 38, 42))
                .rounding(10.0),
        );

    win.show(ctx, |ui| {
        ui.add_space(4.0);
        ui.label(
            RichText::new(i18n::settings_section_language(loc))
                .size(17.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.label(
            RichText::new(i18n::settings_language_help(loc))
                .small()
                .color(Color32::GRAY),
        );
        ui.add_space(8.0);

        let mut lang = settings.ui_language.clone();
        egui::ComboBox::from_id_salt("tonet_ui_lang")
            .width(280.0)
            .selected_text(match lang.as_str() {
                "en" => i18n::lang_option_en(loc),
                "es" => i18n::lang_option_es(loc),
                "de" => i18n::lang_option_de(loc),
                "fr" => i18n::lang_option_fr(loc),
                _ => i18n::lang_option_auto(loc),
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut lang, "auto".to_string(), i18n::lang_option_auto(loc));
                ui.selectable_value(&mut lang, "en".to_string(), i18n::lang_option_en(loc));
                ui.selectable_value(&mut lang, "es".to_string(), i18n::lang_option_es(loc));
                ui.selectable_value(&mut lang, "de".to_string(), i18n::lang_option_de(loc));
                ui.selectable_value(&mut lang, "fr".to_string(), i18n::lang_option_fr(loc));
            });
        settings.ui_language = lang;

        ui.add_space(16.0);
        ui.separator();
        ui.add_space(10.0);

        ui.label(
            RichText::new(i18n::settings_section_updates(loc))
                .size(17.0)
                .strong()
                .color(Color32::WHITE),
        );
        ui.add_space(6.0);
        ui.label(
            RichText::new(i18n::installed_version(loc, current_version))
                .small()
                .color(Color32::LIGHT_GRAY),
        );
        ui.add_space(12.0);

        ui.label(
            RichText::new(i18n::update_policy_question(loc)).color(Color32::from_gray(220)),
        );
        ui.add_space(10.0);

        for policy in [
            UpdatePolicy::OnStartup,
            UpdatePolicy::Periodic,
            UpdatePolicy::ManualOnly,
        ] {
            let label = i18n::update_policy_label(loc, policy);
            let help = i18n::update_policy_help(loc, policy);
            ui.radio_value(&mut settings.update_policy, policy, label)
                .on_hover_text(help);
            ui.label(RichText::new(help).small().color(Color32::GRAY).italics());
            ui.add_space(8.0);
        }

        ui.separator();
        ui.add_space(8.0);

        ui.horizontal(|ui| {
            let can_check = !update_busy;
            let r = ui.add_enabled(
                can_check,
                egui::Button::new(RichText::new(i18n::check_now(loc)).strong()),
            );
            if r.clicked() {
                on_check_now();
            }
            if !can_check {
                r.on_disabled_hover_text(i18n::check_busy_hover(loc));
            }
            if update_busy {
                ui.spinner();
                ui.label(
                    RichText::new(i18n::checking(loc))
                        .small()
                        .color(Color32::GRAY),
                );
            }
        });

        ui.add_space(8.0);
        if !status_line.is_empty() {
            egui::Frame::default()
                .fill(Color32::from_rgb(30, 32, 36))
                .inner_margin(10.0)
                .rounding(6.0)
                .show(ui, |ui| {
                    ui.label(RichText::new(status_line).color(Color32::from_gray(210)));
                });
        }

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button(i18n::save_preferences(loc)).clicked() {
                on_save(settings);
            }
            if ui
                .button(i18n::open_downloads_page(loc))
                .on_hover_text(i18n::open_downloads_tooltip(loc))
                .clicked()
            {
                crate::update::open_downloads_page();
            }
        });
    });
}
