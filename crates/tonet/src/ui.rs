//! Browser chrome (toolbar, omnibox, settings) — layout inspired by mainstream browsers.

use egui::{Align, Color32, Layout, RichText, Ui, Vec2};

use crate::i18n::Locale;
use crate::i18n;
use crate::settings::{AppSettings, UpdatePolicy};

/// Result of the main toolbar (omnibox row).
pub struct ToolbarResult {
    pub navigate: bool,
    pub reload: bool,
    pub open_settings: bool,
    pub go_back: bool,
    pub go_forward: bool,
}

/// Chromium-style row: back / forward / reload, security chip, URL, Go, settings.
pub fn show_chrome_toolbar(
    ui: &mut Ui,
    loc: Locale,
    url_input: &mut String,
    loading: bool,
    can_back: bool,
    can_forward: bool,
) -> ToolbarResult {
    let mut navigate = false;
    let mut reload = false;
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

                let b_reload = ui
                    .add(egui::Button::new(RichText::new("↻").size(18.0)).min_size(btn_size))
                    .on_hover_text(i18n::reload_tooltip(loc));
                if b_reload.clicked() {
                    reload = true;
                }

                ui.separator();

                ui.add(
                    egui::Label::new(
                        RichText::new(format!("🔒  {}", i18n::security_chip_placeholder(loc)))
                            .small()
                            .color(Color32::from_gray(180)),
                    )
                    .truncate(),
                )
                .on_hover_text(
                    "TLS and page trust indicators will grow with Tonet’s security roadmap.",
                );

                let url_w = (ui.available_width() - 130.0).max(80.0);
                let url_response = ui.add_sized(
                    [url_w, 28.0],
                    egui::TextEdit::singleline(url_input)
                        .hint_text(i18n::address_hint(loc))
                        .desired_rows(1),
                );
                if url_response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
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
