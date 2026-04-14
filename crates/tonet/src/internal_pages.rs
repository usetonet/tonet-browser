//! Internal `tonet://` pages (Settings, Downloads, History) — Brave-style chrome pages.

use std::collections::HashSet;

use chrono::{Duration, TimeZone, Utc};
use egui::{Color32, RichText, ScrollArea, Stroke, Ui, Vec2};

use crate::browser_log::{BrowserLog, DownloadRecord, VisitRecord};
use crate::i18n::{self, Locale};
use crate::settings::AppSettings;
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

pub fn parse_tonet_url(url: &str) -> Option<InternalRoute> {
    let t = url.trim().to_ascii_lowercase();
    let rest = t.strip_prefix("tonet://")?;
    let host = rest
        .split(|c| matches!(c, '/' | '?' | '#'))
        .next()
        .filter(|s| !s.is_empty())?
        .to_ascii_lowercase();
    match host.as_str() {
        "settings" => Some(InternalRoute::Settings),
        "downloads" => Some(InternalRoute::Downloads),
        "history" => Some(InternalRoute::History),
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

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsNav {
    #[default]
    General,
    Search,
    Updates,
}

fn settings_sidebar(ui: &mut Ui, loc: Locale, nav: &mut SettingsNav) {
    ui.set_width(SIDEBAR_W);
    egui::Frame::none()
        .fill(theme::OMNIBOX_FILL)
        .stroke(Stroke::new(1.0, theme::OMNIBOX_STROKE))
        .rounding(10.0)
        .inner_margin(10.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 4.0;
                let items = [
                    (SettingsNav::General, i18n::internal_settings_nav_general(loc)),
                    (SettingsNav::Search, i18n::internal_settings_nav_search(loc)),
                    (SettingsNav::Updates, i18n::internal_settings_nav_updates(loc)),
                ];
                for (item, label) in items {
                    let sel = *nav == item;
                    if ui.selectable_label(sel, label).clicked() {
                        *nav = item;
                    }
                }
            });
        });
}

#[allow(clippy::too_many_arguments)]
pub fn show_settings_page(
    ui: &mut Ui,
    loc: Locale,
    current: InternalRoute,
    nav: &mut SettingsNav,
    settings: &mut AppSettings,
    update_busy: bool,
    status_line: &str,
    current_version: &str,
    mut on_save: impl FnMut(&AppSettings),
    mut on_check_now: impl FnMut(),
) -> InternalPageOutput {
    let mut out = InternalPageOutput { navigate_to: None };
    ui.vertical(|ui| {
        if let Some(r) = top_route_tabs(ui, loc, current) {
            out.navigate_to = Some(r.canonical_url().to_string());
        }
        ui.add_space(12.0);
        ui.horizontal_top(|ui| {
            settings_sidebar(ui, loc, nav);
            ui.add_space(12.0);
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::Frame::none()
                        .fill(CARD)
                        .stroke(Stroke::new(1.0, CARD_STROKE))
                        .rounding(12.0)
                        .inner_margin(16.0)
                        .show(ui, |ui| {
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
                            match *nav {
                                SettingsNav::General => {
                                    crate::ui::render_settings_language_section(
                                        ui, settings, loc, form_id,
                                    );
                                }
                                SettingsNav::Search => {
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
                                    out.navigate_to =
                                        Some(InternalRoute::Downloads.canonical_url().to_string());
                                }
                            });
                        });
                });
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tonet_hosts() {
        assert_eq!(
            parse_tonet_url("tonet://settings"),
            Some(InternalRoute::Settings)
        );
        assert_eq!(
            parse_tonet_url("TONET://DOWNLOADS/extra"),
            Some(InternalRoute::Downloads)
        );
        assert_eq!(parse_tonet_url("tonet://history"), Some(InternalRoute::History));
        assert!(parse_tonet_url("tonet://unknown").is_none());
    }
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
