//! Shared UI panels: error, loading, update banner, settings window, and the omnibox ID helper.

use egui::{
    Align, Color32, Id, Layout, RichText, Stroke, Ui,
};

use crate::i18n::{self, Locale};
use crate::settings::{AppSettings, SearchEngine, UpdatePolicy};
use crate::theme;

/// Stable [`Id`] for the omnibox so shortcuts can request focus and selection.
#[inline]
pub fn omnibox_id() -> Id {
    Id::new("tonet_omnibox")
}

pub fn show_error_panel(ui: &mut Ui, loc: Locale, message: &str) {
    egui::Frame::default()
        .fill(theme::ERROR_BG)
        .stroke(Stroke::new(1.0, theme::ERROR_STROKE))
        .inner_margin(egui::Margin::symmetric(18.0, 14.0))
        .rounding(12.0)
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.label(
                    RichText::new("!")
                        .strong()
                        .size(20.0)
                        .color(theme::ERROR_TITLE),
                );
                ui.add_space(10.0);
                ui.with_layout(Layout::top_down(Align::Min), |ui| {
                    ui.label(
                        RichText::new(i18n::error_title(loc))
                            .strong()
                            .size(15.0)
                            .color(theme::ERROR_TITLE),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(message)
                            .size(14.0)
                            .color(theme::ERROR_BODY),
                    );
                });
            });
        });
}

pub fn show_loading(ui: &mut Ui, loc: Locale) {
    ui.vertical_centered(|ui| {
        ui.add_space(28.0);
        egui::Frame::default()
            .fill(theme::OMNIBOX_FILL)
            .stroke(Stroke::new(1.0, theme::OMNIBOX_STROKE))
            .inner_margin(egui::Margin::symmetric(28.0, 22.0))
            .rounding(14.0)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.spinner();
                    ui.add_space(12.0);
                    ui.label(
                        RichText::new(i18n::loading_title(loc))
                            .size(17.0)
                            .strong()
                            .color(theme::TAB_TEXT),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(i18n::loading_sub(loc))
                            .small()
                            .color(theme::LOADING_MUTED),
                    );
                });
            });
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
        .fill(theme::UPDATE_BANNER_BG)
        .stroke(Stroke::new(1.0, theme::UPDATE_BANNER_STROKE))
        .inner_margin(14.0)
        .rounding(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(i18n::update_banner_title(loc))
                        .strong()
                        .color(theme::SETTINGS_HEADING),
                );
                ui.label(
                    RichText::new(version_label)
                        .strong()
                        .color(theme::UPDATE_ACCENT_LABEL),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui.button(i18n::update_dismiss(loc)).clicked() {
                        on_dismiss();
                    }
                    if ui
                        .add(
                            egui::Button::new(RichText::new(i18n::update_download(loc)).strong())
                                .rounding(7.0)
                                .fill(theme::PRIMARY_BTN),
                        )
                        .clicked()
                    {
                        on_open_downloads();
                    }
                });
            });
        });
}

/// Language UI block (also used on `tonet://settings`).
pub fn render_settings_language_section(
    ui: &mut Ui,
    settings: &mut AppSettings,
    loc: Locale,
    form_id: Id,
) {
    ui.label(
        RichText::new(i18n::settings_section_language(loc))
            .size(17.0)
            .strong()
            .color(theme::SETTINGS_HEADING),
    );
    ui.label(
        RichText::new(i18n::settings_language_help(loc))
            .small()
            .color(Color32::GRAY),
    );
    ui.add_space(8.0);

    let mut lang = settings.ui_language.clone();
    egui::ComboBox::from_id_salt(form_id.with("lang"))
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
}

/// Default search engine block (`tonet://settings` + modal).
pub fn render_settings_search_section(
    ui: &mut Ui,
    settings: &mut AppSettings,
    loc: Locale,
    form_id: Id,
) {
    ui.label(
        RichText::new(i18n::settings_section_search(loc))
            .size(17.0)
            .strong()
            .color(theme::SETTINGS_HEADING),
    );
    ui.label(
        RichText::new(i18n::settings_search_help(loc))
            .small()
            .color(Color32::GRAY),
    );
    ui.add_space(10.0);

    for (i, engine) in [
        SearchEngine::Duckduckgo,
        SearchEngine::Google,
        SearchEngine::Brave,
    ]
    .into_iter()
    .enumerate()
    {
        let label = i18n::search_engine_label(loc, engine);
        let help = i18n::search_engine_help(loc, engine);
        ui.push_id(form_id.with(("se", i)), |ui| {
            ui.radio_value(&mut settings.search_engine, engine, label)
                .on_hover_text(help);
        });
        ui.label(RichText::new(help).small().color(Color32::GRAY).italics());
        ui.add_space(8.0);
    }
}

/// Updates policy + check controls (`tonet://settings` + modal).
#[allow(clippy::too_many_arguments)]
pub fn render_settings_updates_section(
    ui: &mut Ui,
    settings: &mut AppSettings,
    loc: Locale,
    form_id: Id,
    update_busy: bool,
    status_line: &str,
    current_version: &str,
    mut on_check_now: impl FnMut(),
) {
    ui.label(
        RichText::new(i18n::settings_section_updates(loc))
            .size(17.0)
            .strong()
            .color(theme::SETTINGS_HEADING),
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

    for (i, policy) in [
        UpdatePolicy::OnStartup,
        UpdatePolicy::Periodic,
        UpdatePolicy::ManualOnly,
    ]
    .into_iter()
    .enumerate()
    {
        let label = i18n::update_policy_label(loc, policy);
        let help = i18n::update_policy_help(loc, policy);
        ui.push_id(form_id.with(("up", i)), |ui| {
            ui.radio_value(&mut settings.update_policy, policy, label)
                .on_hover_text(help);
        });
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
            .fill(theme::SETTINGS_STATUS_BG)
            .inner_margin(10.0)
            .rounding(8.0)
            .show(ui, |ui| {
                ui.label(RichText::new(status_line).color(theme::CHIP));
            });
    }
}

/// Full stacked settings form (modal window).
#[allow(clippy::too_many_arguments)]
pub fn render_settings_form_full(
    ui: &mut Ui,
    settings: &mut AppSettings,
    loc: Locale,
    form_id: Id,
    update_busy: bool,
    status_line: &str,
    current_version: &str,
    mut on_save: impl FnMut(&AppSettings),
    mut on_check_now: impl FnMut(),
) {
    ui.add_space(4.0);
    render_settings_language_section(ui, settings, loc, form_id);

    ui.add_space(16.0);
    ui.separator();
    ui.add_space(10.0);

    render_settings_search_section(ui, settings, loc, form_id);

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(10.0);

    render_settings_updates_section(
        ui,
        settings,
        loc,
        form_id,
        update_busy,
        status_line,
        current_version,
        || on_check_now(),
    );

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
                .fill(theme::SETTINGS_WINDOW_BG)
                .rounding(12.0),
        );

    win.show(ctx, |ui| {
        render_settings_form_full(
            ui,
            settings,
            loc,
            Id::new("tonet_settings_modal"),
            update_busy,
            status_line,
            current_version,
            |s| on_save(s),
            || on_check_now(),
        );
    });
}
