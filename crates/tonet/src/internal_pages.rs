//! Internal `tonet://` pages (Settings, Downloads, History) — Brave-style chrome pages.

use std::collections::HashSet;

use chrono::{Duration, TimeZone, Utc};
use egui::{Color32, RichText, ScrollArea, Stroke, Ui, Vec2};

use crate::browser_log::{BrowserLog, DownloadRecord, VisitRecord};
use crate::i18n::{self, Locale};
use crate::settings::{AppSettings, EnergySaverMode, StartupPolicy};
use crate::shortcut_catalog;
use crate::theme;

const CARD: Color32 = Color32::from_rgb(38, 40, 48);
const CARD_STROKE: Color32 = Color32::from_rgb(52, 55, 62);
const SIDEBAR_W: f32 = 200.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum InternalRoute {
    Settings,
    Downloads,
    History,
}

impl InternalRoute {
    pub fn canonical_url(self) -> &'static str {
        match self {
            InternalRoute::Settings => "tonet://settings",
            InternalRoute::Downloads => "tonet://downloads",
            InternalRoute::History => "tonet://history",
        }
    }
}

/// Parsed `tonet://` URL including optional settings sub-path.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedTonet {
    pub route: InternalRoute,
    /// Lowercased path starting with `/` (e.g. `/system/shortcuts`). `/` when none.
    pub settings_path: String,
}

impl ParsedTonet {
    /// Canonical URL preserving settings deep links.
    pub fn normalized_url(&self) -> String {
        match self.route {
            InternalRoute::History | InternalRoute::Downloads => self.route.canonical_url().to_string(),
            InternalRoute::Settings => {
                if self.settings_path == "/" {
                    "tonet://settings".to_string()
                } else {
                    format!("tonet://settings{}", self.settings_path)
                }
            }
        }
    }
}

pub fn parse_tonet_url(url: &str) -> Option<ParsedTonet> {
    let lower = url.trim().to_ascii_lowercase();
    let rest = lower.strip_prefix("tonet://")?;
    let host_end = rest
        .find(|c| matches!(c, '/' | '?' | '#'))
        .unwrap_or(rest.len());
    let host = rest.get(..host_end)?.trim();
    if host.is_empty() {
        return None;
    }
    let tail = rest.get(host_end..).unwrap_or("");
    let path_raw = tail
        .split(['?', '#'])
        .next()
        .unwrap_or("")
        .to_string();
    let path_lc = if path_raw.is_empty() || path_raw == "/" {
        "/".to_string()
    } else if path_raw.starts_with('/') {
        path_raw
    } else {
        format!("/{}", path_raw)
    }
    .to_ascii_lowercase();

    match host {
        "settings" => Some(ParsedTonet {
            route: InternalRoute::Settings,
            settings_path: path_lc,
        }),
        "downloads" => Some(ParsedTonet {
            route: InternalRoute::Downloads,
            settings_path: "/".to_string(),
        }),
        "history" => Some(ParsedTonet {
            route: InternalRoute::History,
            settings_path: "/".to_string(),
        }),
        _ => None,
    }
}

pub fn tab_title(route: InternalRoute, loc: Locale) -> &'static str {
    match route {
        InternalRoute::Settings => i18n::internal_tab_title_settings(loc),
        InternalRoute::Downloads => i18n::internal_tab_title_downloads(loc),
        InternalRoute::History => i18n::internal_tab_title_history(loc),
    }
}

pub struct InternalPageOutput {
    pub navigate_to: Option<String>,
}

fn top_route_tabs(ui: &mut Ui, loc: Locale, current: InternalRoute) -> Option<InternalRoute> {
    let mut go = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 4.0;
        for (route, label) in [
            (InternalRoute::History, i18n::internal_nav_history(loc)),
            (InternalRoute::Downloads, i18n::internal_nav_downloads(loc)),
            (InternalRoute::Settings, i18n::internal_nav_settings(loc)),
        ] {
            let selected = route == current;
            let r = ui.selectable_label(selected, label);
            if r.clicked() {
                go = Some(route);
            }
        }
    });
    go
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum SettingsNav {
    #[default]
    GetStarted,
    Appearance,
    Content,
    Shields,
    PrivacySecurity,
    Web3,
    Leo,
    Sync,
    SearchEngine,
    Extensions,
    Autofill,
    Languages,
    DownloadPreferences,
    Accessibility,
    System,
    SystemShortcuts,
    Updates,
    ResetSettings,
}

pub fn settings_nav_from_path(path: &str) -> SettingsNav {
    let p = path.trim().to_ascii_lowercase();
    match p.as_str() {
        "/" | "/get-started" | "/general" => SettingsNav::GetStarted,
        "/appearance" => SettingsNav::Appearance,
        "/content" => SettingsNav::Content,
        "/shields" => SettingsNav::Shields,
        "/privacy-security" => SettingsNav::PrivacySecurity,
        "/web3" => SettingsNav::Web3,
        "/leo" => SettingsNav::Leo,
        "/sync" => SettingsNav::Sync,
        "/search-engine" | "/search" => SettingsNav::SearchEngine,
        "/extensions" => SettingsNav::Extensions,
        "/autofill" => SettingsNav::Autofill,
        "/languages" | "/language" => SettingsNav::Languages,
        "/download-preferences" => SettingsNav::DownloadPreferences,
        "/accessibility" => SettingsNav::Accessibility,
        "/system/shortcuts" => SettingsNav::SystemShortcuts,
        "/system" => SettingsNav::System,
        "/updates" => SettingsNav::Updates,
        "/reset-settings" => SettingsNav::ResetSettings,
        _ => SettingsNav::GetStarted,
    }
}

fn settings_sidebar_label(loc: Locale, nav: SettingsNav) -> &'static str {
    use SettingsNav::*;
    match (loc, nav) {
        (_, GetStarted) => i18n::internal_settings_nav_general(loc),
        (_, SearchEngine) => i18n::internal_settings_nav_search(loc),
        (_, Updates) => i18n::internal_settings_nav_updates(loc),
        (Locale::Es, Appearance) => "Aspecto",
        (Locale::De, Appearance) => "Darstellung",
        (Locale::Fr, Appearance) => "Apparence",
        (Locale::En, Appearance) => "Appearance",
        (Locale::Es, Content) => "Contenido",
        (Locale::De, Content) => "Inhalt",
        (Locale::Fr, Content) => "Contenu",
        (Locale::En, Content) => "Content",
        (Locale::Es, Shields) => "Escudos",
        (Locale::De, Shields) => "Shields",
        (Locale::Fr, Shields) => "Boucliers",
        (Locale::En, Shields) => "Shields",
        (Locale::Es, PrivacySecurity) => "Privacidad y seguridad",
        (Locale::De, PrivacySecurity) => "Datenschutz und Sicherheit",
        (Locale::Fr, PrivacySecurity) => "Confidentialité et sécurité",
        (Locale::En, PrivacySecurity) => "Privacy and security",
        (Locale::Es, Web3) => "Web3",
        (Locale::De, Web3) => "Web3",
        (Locale::Fr, Web3) => "Web3",
        (Locale::En, Web3) => "Web3",
        (Locale::Es, Leo) => "Leo",
        (Locale::De, Leo) => "Leo",
        (Locale::Fr, Leo) => "Leo",
        (Locale::En, Leo) => "Leo",
        (Locale::Es, Sync) => "Sincronización",
        (Locale::De, Sync) => "Synchronisation",
        (Locale::Fr, Sync) => "Synchronisation",
        (Locale::En, Sync) => "Sync",
        (Locale::Es, Extensions) => "Extensiones",
        (Locale::De, Extensions) => "Erweiterungen",
        (Locale::Fr, Extensions) => "Extensions",
        (Locale::En, Extensions) => "Extensions",
        (Locale::Es, Autofill) => "Autocompletar y contraseñas",
        (Locale::De, Autofill) => "Autofill und Passwörter",
        (Locale::Fr, Autofill) => "Saisie automatique et mots de passe",
        (Locale::En, Autofill) => "Autofill and passwords",
        (Locale::Es, Languages) => "Idiomas",
        (Locale::De, Languages) => "Sprachen",
        (Locale::Fr, Languages) => "Langues",
        (Locale::En, Languages) => "Languages",
        (Locale::Es, DownloadPreferences) => "Descargas",
        (Locale::De, DownloadPreferences) => "Downloads",
        (Locale::Fr, DownloadPreferences) => "Téléchargements",
        (Locale::En, DownloadPreferences) => "Downloads",
        (Locale::Es, Accessibility) => "Accesibilidad",
        (Locale::De, Accessibility) => "Bedienungshilfen",
        (Locale::Fr, Accessibility) => "Accessibilité",
        (Locale::En, Accessibility) => "Accessibility",
        (Locale::Es, System) => "Sistema",
        (Locale::De, System) => "System",
        (Locale::Fr, System) => "Système",
        (Locale::En, System) => "System",
        (Locale::Es, SystemShortcuts) => "Atajos",
        (Locale::De, SystemShortcuts) => "Tastenkürzel",
        (Locale::Fr, SystemShortcuts) => "Raccourcis",
        (Locale::En, SystemShortcuts) => "Shortcuts",
        (Locale::Es, ResetSettings) => "Restablecer ajustes",
        (Locale::De, ResetSettings) => "Einstellungen zurücksetzen",
        (Locale::Fr, ResetSettings) => "Réinitialiser les paramètres",
        (Locale::En, ResetSettings) => "Reset settings",
    }
}

pub fn settings_page_url(nav: SettingsNav) -> String {
    let path = match nav {
        SettingsNav::GetStarted => return "tonet://settings".to_string(),
        SettingsNav::Appearance => "/appearance",
        SettingsNav::Content => "/content",
        SettingsNav::Shields => "/shields",
        SettingsNav::PrivacySecurity => "/privacy-security",
        SettingsNav::Web3 => "/web3",
        SettingsNav::Leo => "/leo",
        SettingsNav::Sync => "/sync",
        SettingsNav::SearchEngine => "/search-engine",
        SettingsNav::Extensions => "/extensions",
        SettingsNav::Autofill => "/autofill",
        SettingsNav::Languages => "/languages",
        SettingsNav::DownloadPreferences => "/download-preferences",
        SettingsNav::Accessibility => "/accessibility",
        SettingsNav::System => "/system",
        SettingsNav::SystemShortcuts => "/system/shortcuts",
        SettingsNav::Updates => "/updates",
        SettingsNav::ResetSettings => "/reset-settings",
    };
    format!("tonet://settings{}", path)
}

fn settings_sidebar(
    ui: &mut Ui,
    loc: Locale,
    current: SettingsNav,
    out: &mut InternalPageOutput,
) {
    egui::Frame::none()
        .fill(theme::OMNIBOX_FILL)
        .stroke(Stroke::new(1.0, theme::OMNIBOX_STROKE))
        .rounding(10.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ScrollArea::vertical()
                .max_height(ui.available_height())
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.y = 2.0;
                    let primary = [
                        SettingsNav::GetStarted,
                        SettingsNav::Appearance,
                        SettingsNav::Content,
                        SettingsNav::Shields,
                        SettingsNav::PrivacySecurity,
                        SettingsNav::Web3,
                        SettingsNav::Leo,
                        SettingsNav::Sync,
                        SettingsNav::SearchEngine,
                        SettingsNav::Extensions,
                    ];
                    for nav in primary {
                        let label = settings_sidebar_label(loc, nav);
                        let sel = current == nav;
                        if ui.selectable_label(sel, label).clicked() {
                            out.navigate_to = Some(settings_page_url(nav));
                        }
                    }
                    ui.add_space(6.0);
                    ui.separator();
                    ui.add_space(6.0);
                    let secondary = [
                        SettingsNav::Autofill,
                        SettingsNav::Languages,
                        SettingsNav::DownloadPreferences,
                        SettingsNav::Accessibility,
                        SettingsNav::System,
                        SettingsNav::Updates,
                        SettingsNav::ResetSettings,
                    ];
                    for nav in secondary {
                        let label = settings_sidebar_label(loc, nav);
                        let sel = current == nav;
                        if ui.selectable_label(sel, label).clicked() {
                            out.navigate_to = Some(settings_page_url(nav));
                        }
                    }
                });
        });
}

fn settings_placeholder(ui: &mut Ui, _loc: Locale, title: &str, body: &str) {
    ui.label(
        RichText::new(title)
            .size(16.0)
            .strong()
            .color(theme::SETTINGS_HEADING),
    );
    ui.add_space(6.0);
    ui.label(
        RichText::new(body)
            .small()
            .color(theme::LOADING_MUTED),
    );
}

fn settings_row_nav(ui: &mut Ui, label: &str, hint: &str, out: &mut InternalPageOutput, url: String) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(label).strong().color(theme::TAB_TEXT));
            ui.label(RichText::new(hint).small().color(theme::LOADING_MUTED));
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("›").clicked() {
                out.navigate_to = Some(url);
            }
        });
    });
}

fn render_get_started(ui: &mut Ui, loc: Locale, settings: &mut AppSettings, form_id: egui::Id) {
    ui.label(
        RichText::new(i18n::internal_settings_get_started_heading(loc))
            .size(16.0)
            .strong()
            .color(theme::SETTINGS_HEADING),
    );
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(
                RichText::new(i18n::internal_settings_profile_row(loc))
                    .strong()
                    .color(theme::TAB_TEXT),
            );
            ui.label(
                RichText::new(i18n::internal_settings_profile_hint(loc))
                    .small()
                    .color(theme::LOADING_MUTED),
            );
        });
    });
    ui.add_space(10.0);
    ui.separator();
    ui.add_space(10.0);
    ui.label(
        RichText::new(i18n::internal_settings_startup_heading(loc))
            .strong()
            .color(theme::SETTINGS_HEADING),
    );
    ui.add_space(8.0);
    for (i, pol) in [
        StartupPolicy::NewTabPage,
        StartupPolicy::RestoreSession,
        StartupPolicy::OpenSpecificPages,
    ]
    .into_iter()
    .enumerate()
    {
        let label = i18n::internal_settings_startup_label(loc, pol);
        let help = i18n::internal_settings_startup_help(loc, pol);
        ui.push_id(form_id.with(("su", i)), |ui| {
            ui.radio_value(&mut settings.startup_policy, pol, label)
                .on_hover_text(help);
        });
        ui.label(RichText::new(help).small().color(theme::LOADING_MUTED).italics());
        ui.add_space(6.0);
    }
}

fn render_system_page(
    ui: &mut Ui,
    loc: Locale,
    settings: &mut AppSettings,
    out: &mut InternalPageOutput,
) {
    ui.label(
        RichText::new(i18n::internal_settings_system_heading(loc))
            .size(16.0)
            .strong()
            .color(theme::SETTINGS_HEADING),
    );
    ui.add_space(10.0);
    settings_row_nav(
        ui,
        i18n::internal_settings_shortcuts_row(loc),
        i18n::internal_settings_shortcuts_row_hint(loc),
        out,
        settings_page_url(SettingsNav::SystemShortcuts),
    );
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
    let s = &mut settings.system;
    toggle_row(ui, i18n::internal_settings_bg_apps(loc), &mut s.continue_background_apps);
    toggle_row(
        ui,
        i18n::internal_settings_hw_accel(loc),
        &mut s.use_hardware_acceleration,
    );
    ui.horizontal(|ui| {
        ui.label(RichText::new(i18n::internal_settings_proxy(loc)).color(theme::TAB_TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button(i18n::internal_settings_open_system(loc)).clicked() {
                #[cfg(windows)]
                {
                    let _ = webbrowser::open("ms-settings:network-proxy");
                }
                #[cfg(target_os = "macos")]
                {
                    let _ = webbrowser::open(
                        "x-apple.systempreferences:com.apple.preference.network",
                    );
                }
                #[cfg(all(not(windows), not(target_os = "macos")))]
                {
                    let _ = webbrowser::open("https://duckduckgo.com/?q=system+proxy+settings");
                }
            }
        });
    });
    ui.add_space(6.0);
    toggle_row(
        ui,
        i18n::internal_settings_close_last_tab(loc),
        &mut s.close_window_when_last_tab,
    );
    toggle_row(
        ui,
        i18n::internal_settings_warn_close(loc),
        &mut s.warn_before_closing_multi_tab_window,
    );
    toggle_row(
        ui,
        i18n::internal_settings_fullscreen_hint(loc),
        &mut s.show_fullscreen_esc_reminder,
    );
    ui.add_space(16.0);
    ui.label(
        RichText::new(i18n::internal_settings_vpn_heading(loc))
            .strong()
            .color(theme::SETTINGS_HEADING),
    );
    ui.add_space(8.0);
    toggle_row(ui, i18n::internal_settings_vpn_wireguard(loc), &mut s.vpn_use_wireguard);
    toggle_row(ui, i18n::internal_settings_vpn_tray(loc), &mut s.vpn_show_tray_icon);
    ui.label(
        RichText::new(i18n::internal_settings_vpn_tray_hint(loc))
            .small()
            .color(theme::LOADING_MUTED),
    );
    ui.add_space(16.0);
    ui.label(
        RichText::new(i18n::internal_settings_memory_heading(loc))
            .strong()
            .color(theme::SETTINGS_HEADING),
    );
    ui.add_space(6.0);
    ui.label(
        RichText::new(i18n::internal_settings_memory_body(loc))
            .small()
            .color(theme::LOADING_MUTED),
    );
    ui.add_space(8.0);
    toggle_row(ui, i18n::internal_settings_memory_saver(loc), &mut s.memory_saver_enabled);
    ui.add_space(8.0);
    ui.label(
        RichText::new(i18n::internal_settings_keep_sites(loc))
            .strong()
            .color(theme::TAB_TEXT),
    );
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(i18n::internal_settings_keep_sites_hint(loc))
                .small()
                .color(theme::LOADING_MUTED),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let _ = ui.small_button(i18n::internal_settings_add(loc));
        });
    });
    ui.label(
        RichText::new(i18n::internal_settings_no_sites(loc))
            .small()
            .italics()
            .color(theme::LOADING_MUTED),
    );
    ui.add_space(16.0);
    ui.label(
        RichText::new(i18n::internal_settings_power_heading(loc))
            .strong()
            .color(theme::SETTINGS_HEADING),
    );
    ui.add_space(6.0);
    ui.label(
        RichText::new(i18n::internal_settings_power_body(loc))
            .small()
            .color(theme::LOADING_MUTED),
    );
    ui.add_space(8.0);
    ui.horizontal(|ui| {
        ui.label(RichText::new(i18n::internal_settings_energy_saver(loc)).strong());
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(&mut s.energy_saver_enabled, "");
        });
    });
    if s.energy_saver_enabled {
        ui.add_space(6.0);
        for (i, m) in [
            EnergySaverMode::WhenBatteryLow,
            EnergySaverMode::WhenUnplugged,
        ]
        .into_iter()
        .enumerate()
        {
            ui.push_id(egui::Id::new(("es", i)), |ui| {
                ui.radio_value(&mut s.energy_saver_mode, m, i18n::internal_settings_energy_mode(loc, m));
            });
        }
    }
}

fn toggle_row(ui: &mut Ui, label: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).color(theme::TAB_TEXT));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.checkbox(value, "");
        });
    });
    ui.add_space(4.0);
}

fn render_shortcuts_page(
    ui: &mut Ui,
    loc: Locale,
    out: &mut InternalPageOutput,
    filter: &mut String,
) {
    ui.horizontal(|ui| {
        if ui.button(i18n::internal_settings_shortcuts_back(loc)).clicked() {
            out.navigate_to = Some(settings_page_url(SettingsNav::System));
        }
        ui.label(
            RichText::new(i18n::internal_settings_shortcuts_title(loc))
                .strong()
                .size(16.0)
                .color(theme::SETTINGS_HEADING),
        );
    });
    ui.add_space(8.0);
    ui.label(
        RichText::new(i18n::internal_settings_shortcuts_search_hint(loc))
            .small()
            .color(theme::LOADING_MUTED),
    );
    ui.add_space(6.0);
    ui.add(
        egui::TextEdit::singleline(filter)
            .desired_width(ui.available_width().min(480.0))
            .hint_text(i18n::internal_settings_shortcuts_filter_hint(loc)),
    );
    ui.add_space(10.0);
    let rows = shortcut_catalog::filter_pairs(filter);
    ScrollArea::vertical()
        .max_height(ui.available_height().max(120.0))
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (cmd, keys) in rows {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(RichText::new(&cmd).strong().color(theme::TAB_TEXT));
                        ui.label(RichText::new(&keys).small().color(theme::LOADING_MUTED));
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let _ = ui
                            .small_button(i18n::internal_settings_add(loc))
                            .on_hover_text(i18n::internal_settings_add_hint(loc));
                    });
                });
                ui.separator();
            }
        });
    ui.add_space(10.0);
    ui.label(
        RichText::new(i18n::internal_settings_shortcuts_footer_note(loc))
            .small()
            .color(theme::LOADING_MUTED),
    );
}

fn render_reset_page(ui: &mut Ui, loc: Locale, confirm_reset: &mut bool) {
    ui.label(
        RichText::new(i18n::internal_settings_reset_heading(loc))
            .strong()
            .size(16.0)
            .color(theme::SETTINGS_HEADING),
    );
    ui.add_space(8.0);
    ui.label(
        RichText::new(i18n::internal_settings_reset_body(loc))
            .small()
            .color(theme::LOADING_MUTED),
    );
    ui.add_space(16.0);
    if ui
        .button(RichText::new(i18n::internal_settings_reset_button(loc)).color(Color32::from_rgb(255, 120, 120)))
        .clicked()
    {
        *confirm_reset = true;
    }
}

#[allow(clippy::too_many_arguments)]
pub fn show_settings_page(
    ui: &mut Ui,
    loc: Locale,
    current: InternalRoute,
    settings_url: &str,
    settings: &mut AppSettings,
    update_busy: bool,
    status_line: &str,
    current_version: &str,
    shortcuts_filter: &mut String,
    confirm_reset: &mut bool,
    mut on_save: impl FnMut(&AppSettings),
    mut on_check_now: impl FnMut(),
) -> InternalPageOutput {
    let mut out = InternalPageOutput { navigate_to: None };
    let parsed = parse_tonet_url(settings_url);
    let path = parsed
        .as_ref()
        .map(|p| p.settings_path.as_str())
        .unwrap_or("/");
    let nav = settings_nav_from_path(path);

    ui.vertical(|ui| {
        if let Some(r) = top_route_tabs(ui, loc, current) {
            out.navigate_to = Some(r.canonical_url().to_string());
        }
        ui.add_space(12.0);
        let full_w = ui.available_width();
        let h = ui.available_height();
        let left_w = SIDEBAR_W.min(full_w * 0.34).max(168.0);
        let gap = 12.0;
        let right_w = (full_w - left_w - gap).max(220.0);

        ui.horizontal(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(left_w, h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    settings_sidebar(ui, loc, nav, &mut out);
                },
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                Vec2::new(right_w, h),
                egui::Layout::top_down(egui::Align::Min),
                |ui| {
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            egui::Frame::none()
                                .fill(CARD)
                                .stroke(Stroke::new(1.0, CARD_STROKE))
                                .rounding(12.0)
                                .inner_margin(16.0)
                                .show(ui, |ui| {
                                    ui.vertical(|ui| {
                                        ui.label(
                                            RichText::new(i18n::internal_tab_title_settings(loc))
                                                .size(22.0)
                                                .strong()
                                                .color(theme::SETTINGS_HEADING),
                                        );
                                        ui.add_space(8.0);
                                        ui.label(
                                            RichText::new(i18n::internal_settings_intro(loc))
                                                .small()
                                                .color(theme::LOADING_MUTED),
                                        );
                                        ui.add_space(16.0);
                                        let form_id = egui::Id::new("tonet_settings_internal");
                                        match nav {
                                            SettingsNav::GetStarted => {
                                                render_get_started(ui, loc, settings, form_id);
                                            }
                                            SettingsNav::Languages => {
                                                crate::ui::render_settings_language_section(
                                                    ui, settings, loc, form_id,
                                                );
                                            }
                                            SettingsNav::SearchEngine => {
                                                crate::ui::render_settings_search_section(
                                                    ui, settings, loc, form_id,
                                                );
                                            }
                                            SettingsNav::Updates => {
                                                crate::ui::render_settings_updates_section(
                                                    ui,
                                                    settings,
                                                    loc,
                                                    form_id,
                                                    update_busy,
                                                    status_line,
                                                    current_version,
                                                    || on_check_now(),
                                                );
                                            }
                                            SettingsNav::System => {
                                                render_system_page(ui, loc, settings, &mut out);
                                            }
                                            SettingsNav::SystemShortcuts => {
                                                render_shortcuts_page(ui, loc, &mut out, shortcuts_filter);
                                            }
                                            SettingsNav::ResetSettings => {
                                                render_reset_page(ui, loc, confirm_reset);
                                            }
                                            SettingsNav::Appearance => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_appearance_title(loc),
                                                i18n::internal_settings_appearance_body(loc),
                                            ),
                                            SettingsNav::Content => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_content_title(loc),
                                                i18n::internal_settings_content_body(loc),
                                            ),
                                            SettingsNav::Shields => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_shields_title(loc),
                                                i18n::internal_settings_shields_body(loc),
                                            ),
                                            SettingsNav::PrivacySecurity => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_privacy_title(loc),
                                                i18n::internal_settings_privacy_body(loc),
                                            ),
                                            SettingsNav::Web3 => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_web3_title(loc),
                                                i18n::internal_settings_web3_body(loc),
                                            ),
                                            SettingsNav::Leo => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_leo_title(loc),
                                                i18n::internal_settings_leo_body(loc),
                                            ),
                                            SettingsNav::Sync => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_sync_title(loc),
                                                i18n::internal_settings_sync_body(loc),
                                            ),
                                            SettingsNav::Extensions => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_extensions_title(loc),
                                                i18n::internal_settings_extensions_body(loc),
                                            ),
                                            SettingsNav::Autofill => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_autofill_title(loc),
                                                i18n::internal_settings_autofill_body(loc),
                                            ),
                                            SettingsNav::DownloadPreferences => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_dl_prefs_title(loc),
                                                i18n::internal_settings_dl_prefs_body(loc),
                                            ),
                                            SettingsNav::Accessibility => settings_placeholder(
                                                ui,
                                                loc,
                                                i18n::internal_settings_a11y_title(loc),
                                                i18n::internal_settings_a11y_body(loc),
                                            ),
                                        }
                                        ui.add_space(16.0);
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
                                            if ui.button(i18n::internal_nav_downloads(loc)).clicked() {
                                                out.navigate_to = Some(
                                                    InternalRoute::Downloads
                                                        .canonical_url()
                                                        .to_string(),
                                                );
                                            }
                                        });
                                    });
                                });
                        });
                },
            );
        });
    });
    out
}

fn filter_downloads<'a>(
    items: &'a [DownloadRecord],
    q: &str,
) -> impl Iterator<Item = &'a DownloadRecord> + 'a {
    let q = q.trim().to_ascii_lowercase();
    items.iter().filter(move |d| {
        if q.is_empty() {
            return true;
        }
        d.display_name.to_ascii_lowercase().contains(&q)
            || d.url.to_ascii_lowercase().contains(&q)
    })
}

pub fn show_downloads_page(
    ui: &mut Ui,
    ctx: &egui::Context,
    loc: Locale,
    current: InternalRoute,
    log: &mut BrowserLog,
    search: &mut String,
    confirm_clear: &mut bool,
) -> InternalPageOutput {
    let mut out = InternalPageOutput { navigate_to: None };
    ui.vertical(|ui| {
        if let Some(r) = top_route_tabs(ui, loc, current) {
            out.navigate_to = Some(r.canonical_url().to_string());
        }
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(i18n::internal_tab_title_downloads(loc))
                    .size(22.0)
                    .strong()
                    .color(theme::SETTINGS_HEADING),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(i18n::internal_clear_all(loc)).clicked() {
                    *confirm_clear = true;
                }
                ui.add(
                    egui::TextEdit::singleline(search)
                        .desired_width(220.0)
                        .hint_text(i18n::internal_search_downloads(loc)),
                );
            });
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(i18n::internal_downloads_intro(loc))
                .small()
                .color(theme::LOADING_MUTED),
        );
        ui.add_space(12.0);

        let filtered: Vec<DownloadRecord> =
            filter_downloads(&log.downloads, search).cloned().collect();
        if filtered.is_empty() {
            ui.label(
                RichText::new(i18n::internal_no_items(loc))
                    .italics()
                    .color(theme::LOADING_MUTED),
            );
        } else {
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for d in &filtered {
                        download_row(ui, ctx, loc, d, log, &mut out);
                    }
                });
        }
    });
    out
}

fn download_row(
    ui: &mut Ui,
    ctx: &egui::Context,
    loc: Locale,
    d: &DownloadRecord,
    log: &mut BrowserLog,
    out: &mut InternalPageOutput,
) {
    egui::Frame::none()
        .fill(CARD)
        .stroke(Stroke::new(1.0, CARD_STROKE))
        .rounding(10.0)
        .inner_margin(12.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("⤓").size(18.0).color(theme::ACCENT));
                ui.vertical(|ui| {
                    if ui
                        .link(RichText::new(&d.display_name).strong().color(theme::LINK))
                        .clicked()
                    {
                        out.navigate_to = Some(d.url.clone());
                    }
                    ui.label(
                        RichText::new(format!(
                            "{} {}",
                            i18n::internal_from_url(loc),
                            d.url
                        ))
                        .small()
                        .color(theme::LOADING_MUTED),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("✕").on_hover_text(i18n::internal_remove_row(loc)).clicked()
                    {
                        log.remove_downloads(&[d.id]);
                    }
                    let _ = ui
                        .small_button("📁")
                        .on_hover_text(format!(
                            "{}\n{}",
                            i18n::internal_open_folder(loc),
                            i18n::internal_open_folder_hint(loc)
                        ))
                        .clicked();
                    if ui.small_button("🔗").on_hover_text(i18n::internal_copy_link(loc)).clicked()
                    {
                        ctx.copy_text(d.url.clone());
                    }
                });
            });
        });
    ui.add_space(8.0);
}

fn day_label(ts: i64, loc: Locale) -> String {
    let Some(dt) = Utc.timestamp_opt(ts, 0).single() else {
        return String::new();
    };
    let d = dt.date_naive();
    let today = Utc::now().date_naive();
    if d == today {
        return i18n::internal_hist_today(loc).to_string();
    }
    if let Some(yd) = today.checked_sub_signed(Duration::days(1)) {
        if d == yd {
            return i18n::internal_hist_yesterday(loc).to_string();
        }
    }
    format!("{}", d.format("%A, %e %B %Y"))
}

fn filter_visits<'a>(
    visits: &'a [VisitRecord],
    q: &str,
) -> impl Iterator<Item = &'a VisitRecord> + 'a {
    let q = q.trim().to_ascii_lowercase();
    visits.iter().rev().filter(move |v| {
        if v.url.trim().to_ascii_lowercase().starts_with("tonet://") {
            return false;
        }
        if q.is_empty() {
            return true;
        }
        v.url.to_ascii_lowercase().contains(&q)
            || v
                .title
                .as_ref()
                .map(|t| t.to_ascii_lowercase().contains(&q))
                .unwrap_or(false)
    })
}

pub fn show_history_page(
    ui: &mut Ui,
    ctx: &egui::Context,
    loc: Locale,
    current: InternalRoute,
    log: &mut BrowserLog,
    search: &mut String,
    selected: &mut HashSet<u64>,
    confirm_clear: &mut bool,
) -> InternalPageOutput {
    let mut out = InternalPageOutput { navigate_to: None };
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(SIDEBAR_W);
            egui::Frame::none()
                .fill(theme::OMNIBOX_FILL)
                .stroke(Stroke::new(1.0, theme::OMNIBOX_STROKE))
                .rounding(10.0)
                .inner_margin(10.0)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(i18n::internal_history_sidebar(loc))
                            .strong()
                            .color(theme::ACCENT),
                    );
                    let ir = ui.add_enabled_ui(false, |ui| {
                        ui.selectable_label(
                            false,
                            RichText::new(i18n::internal_history_other_devices(loc))
                                .color(theme::TAB_TEXT_MUTED),
                        )
                    });
                    ir.response
                        .on_hover_text(i18n::internal_history_other_devices_hint(loc));
                    ui.add_space(16.0);
                    if ui.button(i18n::internal_history_delete_data(loc)).clicked() {
                        *confirm_clear = true;
                    }
                });
        });
        ui.add_space(12.0);
        ui.vertical(|ui| {
            if let Some(r) = top_route_tabs(ui, loc, current) {
                out.navigate_to = Some(r.canonical_url().to_string());
            }
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(i18n::internal_tab_title_history(loc))
                        .size(22.0)
                        .strong()
                        .color(theme::SETTINGS_HEADING),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if !selected.is_empty() {
                        if ui.button(i18n::internal_remove_selected(loc)).clicked() {
                            let ids: Vec<u64> = selected.iter().copied().collect();
                            log.remove_visits(&ids);
                            selected.clear();
                        }
                    }
                    ui.add(
                        egui::TextEdit::singleline(search)
                            .desired_width(220.0)
                            .hint_text(i18n::internal_search_history(loc)),
                    );
                });
            });
            ui.add_space(12.0);

            let filtered: Vec<VisitRecord> =
                filter_visits(&log.visits, search).cloned().collect();
            if filtered.is_empty() {
                ui.label(
                    RichText::new(i18n::internal_no_items(loc))
                        .italics()
                        .color(theme::LOADING_MUTED),
                );
            } else {
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut last_day: Option<String> = None;
                        for v in &filtered {
                            let dl = day_label(v.visited_at_unix, loc);
                            if last_day.as_ref() != Some(&dl) {
                                last_day = Some(dl.clone());
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(dl)
                                        .strong()
                                        .color(theme::TAB_TEXT_MUTED),
                                );
                                ui.add_space(6.0);
                            }
                            history_row(ui, ctx, loc, v, selected, log, &mut out);
                        }
                    });
            }
        });
    });
    out
}

fn history_row(
    ui: &mut Ui,
    ctx: &egui::Context,
    loc: Locale,
    v: &VisitRecord,
    selected: &mut HashSet<u64>,
    log: &mut BrowserLog,
    out: &mut InternalPageOutput,
) {
    let time = Utc
        .timestamp_opt(v.visited_at_unix, 0)
        .single()
        .map(|dt| dt.format("%I:%M %p").to_string())
        .unwrap_or_default();
    let title = v
        .title
        .as_deref()
        .filter(|t| !t.trim().is_empty())
        .unwrap_or(&v.url);
    let host = url::Url::parse(&v.url)
        .ok()
        .and_then(|u| u.host_str().map(|s| s.to_string()))
        .unwrap_or_default();

    egui::Frame::none()
        .fill(CARD)
        .stroke(Stroke::new(1.0, CARD_STROKE))
        .rounding(10.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut checked = selected.contains(&v.id);
                if ui.checkbox(&mut checked, "").changed() {
                    if checked {
                        selected.insert(v.id);
                    } else {
                        selected.remove(&v.id);
                    }
                }
                ui.label(
                    RichText::new(time)
                        .small()
                        .color(theme::LOADING_MUTED),
                )
                .on_hover_text(&v.url);
                ui.label("🌐");
                ui.vertical(|ui| {
                    if ui.link(RichText::new(title).color(theme::LINK)).clicked() {
                        out.navigate_to = Some(v.url.clone());
                    }
                    if !host.is_empty() {
                        ui.label(
                            RichText::new(host)
                                .small()
                                .color(theme::LOADING_MUTED),
                        );
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.menu_button("⋮", |ui| {
                        if ui.button(i18n::internal_open_in_tonet(loc)).clicked() {
                            out.navigate_to = Some(v.url.clone());
                            ui.close_menu();
                        }
                        if ui.button(i18n::internal_copy_link(loc)).clicked() {
                            ctx.copy_text(v.url.clone());
                            ui.close_menu();
                        }
                        if ui.button(i18n::internal_remove_row(loc)).clicked() {
                            log.remove_visits(&[v.id]);
                            selected.remove(&v.id);
                            ui.close_menu();
                        }
                    });
                });
            });
        });
    ui.add_space(6.0);
}

pub enum ClearTarget {
    History,
    Downloads,
}

pub fn show_reset_settings_modal(
    ctx: &egui::Context,
    open: &mut bool,
    loc: Locale,
    settings: &mut AppSettings,
) {
    if !*open {
        return;
    }
    egui::Window::new(i18n::internal_settings_reset_confirm_title(loc))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label(i18n::internal_settings_reset_confirm_body(loc));
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(i18n::internal_btn_cancel(loc)).clicked() {
                    *open = false;
                }
                if ui.button(i18n::internal_settings_reset_confirm(loc)).clicked() {
                    *settings = AppSettings::default();
                    let _ = settings.save();
                    *open = false;
                }
            });
        });
}

pub fn show_clear_confirm_modal(
    ctx: &egui::Context,
    open: &mut bool,
    loc: Locale,
    message: &str,
    log: &mut BrowserLog,
    target: ClearTarget,
) {
    if !*open {
        return;
    }
    egui::Window::new(i18n::internal_clear_all(loc))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label(message);
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(i18n::internal_btn_cancel(loc)).clicked() {
                    *open = false;
                }
                if ui.button(i18n::internal_btn_clear(loc)).clicked() {
                    match target {
                        ClearTarget::History => log.clear_visits(),
                        ClearTarget::Downloads => log.clear_downloads(),
                    }
                    *open = false;
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tonet_hosts() {
        let s = parse_tonet_url("tonet://settings").unwrap();
        assert_eq!(s.route, InternalRoute::Settings);
        assert_eq!(s.settings_path, "/");
        assert_eq!(s.normalized_url(), "tonet://settings");

        let d = parse_tonet_url("TONET://DOWNLOADS/extra").unwrap();
        assert_eq!(d.route, InternalRoute::Downloads);

        assert_eq!(parse_tonet_url("tonet://history").unwrap().route, InternalRoute::History);
        assert!(parse_tonet_url("tonet://unknown").is_none());

        let deep = parse_tonet_url("tonet://settings/system/shortcuts").unwrap();
        assert_eq!(deep.route, InternalRoute::Settings);
        assert_eq!(deep.settings_path, "/system/shortcuts");
        assert_eq!(deep.normalized_url(), "tonet://settings/system/shortcuts");
    }
}
