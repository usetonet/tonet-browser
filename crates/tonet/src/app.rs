//! Tonet application: tabs, chrome, navigation history, locale, and update checks.

use std::cell::Cell;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ViewportCommand};

use crate::branding;
use crate::i18n::{self, Locale};
use crate::network::fetch_url;
use crate::parser::parse_html;
use crate::renderer::render_nodes;
use crate::settings::{AppSettings, UpdatePolicy};
use crate::tab::{NavigateIntent, Tab, DEFAULT_HOME_URL};
use crate::ui::{
    show_chrome_toolbar, show_error_panel, show_loading, show_settings_window, show_tab_bar,
    show_update_banner,
};
use crate::update;

#[derive(Debug)]
enum UpdateJobResult {
    UpToDate,
    UpdateAvailable { version: String },
    Error(String),
}

pub struct TonetApp {
    tabs: Vec<Tab>,
    active_tab: usize,
    window_title: String,

    settings: AppSettings,
    settings_open: bool,
    startup_check_done: bool,

    update_rx: Option<mpsc::Receiver<UpdateJobResult>>,
    update_busy: bool,
    update_status_line: String,
    update_banner: Option<String>,
    update_banner_dismissed: bool,

    last_periodic_check: Instant,
}

impl TonetApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.include_bytes(
            branding::TONET_LOGO_URI,
            egui::load::Bytes::Static(branding::TONET_SVG),
        );

        let settings = AppSettings::load();
        Self {
            tabs: vec![Tab::new(DEFAULT_HOME_URL)],
            active_tab: 0,
            window_title: "Tonet".to_string(),
            settings,
            settings_open: false,
            startup_check_done: false,
            update_rx: None,
            update_busy: false,
            update_status_line: String::new(),
            update_banner: None,
            update_banner_dismissed: false,
            last_periodic_check: Instant::now(),
        }
    }

    fn loc(&self) -> Locale {
        i18n::effective_locale(&self.settings)
    }

    fn active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab]
    }

    fn active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab]
    }

    fn tab_strip_title(tab: &Tab, loc: Locale) -> String {
        if let Some(t) = tab.doc_title_trimmed() {
            Self::clamp_strip_label(t, 24)
        } else {
            let u = tab.url_input.trim();
            if u.is_empty() {
                i18n::tab_untitled(loc).to_string()
            } else {
                Self::clamp_strip_label(u, 32)
            }
        }
    }

    fn clamp_strip_label(s: &str, max_chars: usize) -> String {
        let t = s.trim();
        let count = t.chars().count();
        if count <= max_chars {
            return t.to_string();
        }
        t.chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>()
            + "…"
    }

    fn set_active_tab(&mut self, idx: usize, ctx: &egui::Context) {
        if idx >= self.tabs.len() || idx == self.active_tab {
            return;
        }
        // Keep background fetches running; `poll_fetch` delivers results per tab.
        self.active_tab = idx;
        self.sync_window_title(ctx);
    }

    fn open_new_tab(&mut self, ctx: &egui::Context) {
        self.tabs.push(Tab::new(DEFAULT_HOME_URL));
        self.active_tab = self.tabs.len() - 1;
        self.sync_window_title(ctx);
    }

    fn close_tab_at(&mut self, index: usize, ctx: &egui::Context) {
        if self.tabs.len() <= 1 {
            return;
        }
        self.tabs[index].cancel_in_flight();
        self.tabs.remove(index);
        if index < self.active_tab {
            self.active_tab -= 1;
        } else if index == self.active_tab {
            self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
        }
        self.sync_window_title(ctx);
    }

    fn start_fetch_with_intent(&mut self, intent: NavigateIntent) {
        let loc = self.loc();
        let tab = self.active_tab_mut();
        let trimmed = tab.url_input.trim().to_string();
        if trimmed.is_empty() {
            tab.error_message = Some(i18n::err_empty_url(loc).to_string());
            tab.dom.clear();
            return;
        }

        tab.loading = true;
        tab.error_message = None;
        if matches!(intent, NavigateIntent::NewPage) {
            tab.dom.clear();
        }

        tab.pending_nav = Some((trimmed.clone(), intent));

        let (tx, rx) = mpsc::channel();
        tab.fetch_rx = Some(rx);

        let page_url = trimmed.clone();
        std::thread::spawn(move || {
            let outcome = (|| {
                let html = fetch_url(&page_url).map_err(|e| e.to_string())?;
                Ok(parse_html(&html, &page_url))
            })();
            let _ = tx.send(outcome);
        });
    }

    fn start_fetch_new(&mut self) {
        self.start_fetch_with_intent(NavigateIntent::NewPage);
    }

    fn reload_page(&mut self) {
        self.start_fetch_with_intent(NavigateIntent::Reload);
    }

    fn go_back(&mut self, ctx: &egui::Context) {
        let tab = self.active_tab_mut();
        if tab.hist_index == 0 {
            return;
        }
        tab.hist_index -= 1;
        let e = tab.history[tab.hist_index].clone();
        tab.url_input = e.url;
        tab.dom = e.nodes;
        tab.error_message = None;
        self.sync_window_title(ctx);
    }

    fn go_forward(&mut self, ctx: &egui::Context) {
        let tab = self.active_tab_mut();
        if tab.hist_index + 1 >= tab.history.len() {
            return;
        }
        tab.hist_index += 1;
        let e = tab.history[tab.hist_index].clone();
        tab.url_input = e.url;
        tab.dom = e.nodes;
        tab.error_message = None;
        self.sync_window_title(ctx);
    }

    fn sync_window_title(&mut self, ctx: &egui::Context) {
        let tab = self.active_tab();
        let doc_title = tab.doc_title_trimmed();

        let new_title = doc_title.unwrap_or(i18n::app_name(self.loc()));
        if new_title != self.window_title {
            self.window_title = new_title.to_string();
            ctx.send_viewport_cmd(ViewportCommand::Title(self.window_title.clone()));
        }
    }

    fn poll_fetch(&mut self, ctx: &egui::Context) {
        let loc = self.loc();
        let active = self.active_tab;
        let mut title_dirty = false;
        let mut reset_window_title = false;

        for (i, tab) in self.tabs.iter_mut().enumerate() {
            let Some(rx) = &tab.fetch_rx else {
                continue;
            };

            match rx.try_recv() {
                Ok(Ok(nodes)) => {
                    tab.loading = false;
                    tab.fetch_rx = None;
                    tab.apply_successful_navigation(nodes);
                    if i == active {
                        title_dirty = true;
                    }
                    ctx.request_repaint();
                }
                Ok(Err(msg)) => {
                    tab.error_message = Some(i18n::localize_fetch_error(loc, &msg));
                    tab.loading = false;
                    tab.fetch_rx = None;
                    tab.pending_nav = None;
                    if i == active {
                        reset_window_title = true;
                    }
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    tab.loading = false;
                    tab.fetch_rx = None;
                    tab.pending_nav = None;
                    tab.error_message = Some(i18n::err_fetch_disconnected(loc).to_string());
                    ctx.request_repaint();
                }
            }
        }

        if reset_window_title {
            ctx.send_viewport_cmd(ViewportCommand::Title("Tonet".into()));
            self.window_title = "Tonet".to_string();
        } else if title_dirty {
            self.sync_window_title(ctx);
        }
    }

    fn spawn_update_check(&mut self) {
        if self.update_busy {
            return;
        }
        self.update_busy = true;
        self.update_status_line = i18n::update_checking_github(self.loc()).to_string();

        let (tx, rx) = mpsc::channel();
        self.update_rx = Some(rx);

        std::thread::spawn(move || {
            let msg = match update::check_for_newer_release() {
                Ok(Some(ver)) => UpdateJobResult::UpdateAvailable {
                    version: ver.to_string(),
                },
                Ok(None) => UpdateJobResult::UpToDate,
                Err(e) => UpdateJobResult::Error(e.to_string()),
            };
            let _ = tx.send(msg);
        });
    }

    fn poll_update_job(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.update_rx else {
            return;
        };
        let loc = self.loc();

        match rx.try_recv() {
            Ok(result) => {
                self.update_busy = false;
                self.update_rx = None;
                let now = chrono::Utc::now().timestamp();
                self.settings.last_update_check_unix = Some(now);
                let _ = self.settings.save();

                match result {
                    UpdateJobResult::UpToDate => {
                        self.update_status_line =
                            i18n::update_up_to_date(loc, env!("CARGO_PKG_VERSION"));
                    }
                    UpdateJobResult::UpdateAvailable { version } => {
                        self.update_status_line = i18n::update_new_version(loc, &version);
                        if !self.update_banner_dismissed {
                            self.update_banner = Some(version);
                        }
                    }
                    UpdateJobResult::Error(e) => {
                        self.update_status_line = i18n::update_check_failed(loc, &e);
                    }
                }
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Empty) => {
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.update_busy = false;
                self.update_rx = None;
                self.update_status_line = i18n::update_interrupted(loc).to_string();
                ctx.request_repaint();
            }
        }
    }

    fn maybe_schedule_update_checks(&mut self, _ctx: &egui::Context) {
        if self.update_busy || self.update_rx.is_some() {
            return;
        }

        if !self.startup_check_done {
            self.startup_check_done = true;
            if self.settings.update_policy != UpdatePolicy::ManualOnly {
                self.spawn_update_check();
            }
            return;
        }

        if self.settings.update_policy == UpdatePolicy::Periodic {
            if self.last_periodic_check.elapsed() < Duration::from_secs(60) {
                return;
            }
            self.last_periodic_check = Instant::now();

            let now = chrono::Utc::now().timestamp();
            let due = match self.settings.last_update_check_unix {
                None => true,
                Some(t) => now.saturating_sub(t) >= 86_400,
            };
            if due {
                self.spawn_update_check();
            }
        }
    }
}

impl eframe::App for TonetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(url) = self.active_tab_mut().pending_link_navigation.take() {
            self.active_tab_mut().url_input = url;
            self.start_fetch_new();
        }

        self.poll_fetch(ctx);
        self.poll_update_job(ctx);
        self.maybe_schedule_update_checks(ctx);

        if ctx.input(|i| {
            i.key_pressed(egui::Key::F5)
                || (i.modifiers.command && i.key_pressed(egui::Key::R))
        }) && !self.active_tab().loading
        {
            self.reload_page();
        }

        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::T)) {
            self.open_new_tab(ctx);
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::W)) {
            self.close_tab_at(self.active_tab, ctx);
        }

        let loc = self.loc();
        let tab = self.active_tab();
        let can_back = tab.hist_index > 0;
        let can_forward = tab.hist_index + 1 < tab.history.len();
        let tab_titles: Vec<String> = self
            .tabs
            .iter()
            .map(|t| Self::tab_strip_title(t, loc))
            .collect();
        let can_close_tabs = self.tabs.len() > 1;

        egui::TopBottomPanel::top("tonet_top").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.vertical(|ui| {
                if let Some(ver) = &self.update_banner {
                    if !self.update_banner_dismissed {
                        show_update_banner(
                            ui,
                            loc,
                            &format!("v{ver}"),
                            update::open_downloads_page,
                            || {
                                self.update_banner_dismissed = true;
                                self.update_banner = None;
                            },
                        );
                        ui.add_space(6.0);
                    }
                }

                let tb_tabs = show_tab_bar(
                    ui,
                    loc,
                    &tab_titles,
                    self.active_tab,
                    can_close_tabs,
                );
                if tb_tabs.new_tab {
                    self.open_new_tab(ctx);
                }
                if let Some(i) = tb_tabs.select_tab {
                    self.set_active_tab(i, ctx);
                }
                if let Some(i) = tb_tabs.close_tab {
                    self.close_tab_at(i, ctx);
                }
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.add(
                        egui::Image::from_uri(branding::TONET_LOGO_URI)
                            .max_height(22.0)
                            .rounding(6.0),
                    );
                    ui.label(
                        egui::RichText::new(i18n::app_name(loc))
                            .strong()
                            .size(15.0)
                            .color(Color32::from_rgb(120, 175, 255)),
                    );
                    ui.separator();
                });

                let active = self.active_tab_mut();
                let chip_preview = active.url_input.trim().to_string();
                let tb = show_chrome_toolbar(
                    ui,
                    loc,
                    &mut active.url_input,
                    &chip_preview,
                    active.loading,
                    can_back,
                    can_forward,
                );
                if tb.go_back {
                    self.go_back(ctx);
                }
                if tb.go_forward {
                    self.go_forward(ctx);
                }
                if tb.reload {
                    self.reload_page();
                }
                if tb.navigate {
                    self.start_fetch_new();
                }
                if tb.open_settings {
                    self.settings_open = true;
                }
            });
            ui.add_space(4.0);
        });

        let tab = self.active_tab_mut();
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(err) = &tab.error_message {
                show_error_panel(ui, loc, err);
                ui.add_space(8.0);
            }

            if tab.loading {
                show_loading(ui, loc);
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if !tab.loading && tab.error_message.is_none() {
                        render_nodes(ui, loc, &tab.dom, &mut tab.pending_link_navigation);
                    } else if !tab.loading && tab.error_message.is_some() {
                        ui.label(
                            egui::RichText::new(i18n::suggestion_fix_url(loc))
                                .italics()
                                .color(Color32::GRAY),
                        );
                    }
                });
        });

        let current = env!("CARGO_PKG_VERSION");
        let check_now = Cell::new(false);
        show_settings_window(
            ctx,
            &mut self.settings_open,
            &mut self.settings,
            loc,
            self.update_busy,
            &self.update_status_line,
            current,
            |s| {
                let _ = s.save();
            },
            || {
                check_now.set(true);
            },
        );
        if check_now.get() {
            self.update_banner_dismissed = false;
            self.spawn_update_check();
        }

        ctx.input(|i| {
            if i.modifiers.command && i.key_pressed(egui::Key::Comma) {
                self.settings_open = true;
            }
        });
    }
}
