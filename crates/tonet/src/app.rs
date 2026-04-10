//! Tonet application: chrome, navigation history, locale, and update checks.

use std::cell::Cell;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ViewportCommand};

use crate::i18n::{self, Locale};
use crate::network::fetch_url;
use crate::parser::{parse_html, DomNode, DomNodeType};
use crate::renderer::render_nodes;
use crate::settings::{AppSettings, UpdatePolicy};
use crate::ui::{
    show_chrome_toolbar, show_error_panel, show_loading, show_settings_window, show_update_banner,
};
use crate::update;

type FetchResult = Result<Vec<DomNode>, String>;

#[derive(Debug, Clone, Copy)]
enum NavigateIntent {
    NewPage,
    Reload,
}

#[derive(Clone)]
struct HistoryEntry {
    url: String,
    nodes: Vec<DomNode>,
}

#[derive(Debug)]
enum UpdateJobResult {
    UpToDate,
    UpdateAvailable { version: String },
    Error(String),
}

pub struct TonetApp {
    url_input: String,
    loading: bool,
    error_message: Option<String>,
    dom: Vec<DomNode>,
    window_title: String,
    fetch_rx: Option<mpsc::Receiver<FetchResult>>,

    settings: AppSettings,
    settings_open: bool,
    startup_check_done: bool,

    update_rx: Option<mpsc::Receiver<UpdateJobResult>>,
    update_busy: bool,
    update_status_line: String,
    update_banner: Option<String>,
    update_banner_dismissed: bool,

    last_periodic_check: Instant,

    history: Vec<HistoryEntry>,
    hist_index: usize,
    pending_nav: Option<(String, NavigateIntent)>,
}

impl TonetApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let settings = AppSettings::load();
        Self {
            url_input: "https://usetonet.com".to_string(),
            loading: false,
            error_message: None,
            dom: Vec::new(),
            window_title: "Tonet".to_string(),
            fetch_rx: None,
            settings,
            settings_open: false,
            startup_check_done: false,
            update_rx: None,
            update_busy: false,
            update_status_line: String::new(),
            update_banner: None,
            update_banner_dismissed: false,
            last_periodic_check: Instant::now(),
            history: Vec::new(),
            hist_index: 0,
            pending_nav: None,
        }
    }

    fn loc(&self) -> Locale {
        i18n::effective_locale(&self.settings)
    }

    fn start_fetch_with_intent(&mut self, intent: NavigateIntent) {
        let loc = self.loc();
        let trimmed = self.url_input.trim().to_string();
        if trimmed.is_empty() {
            self.error_message = Some(i18n::err_empty_url(loc).to_string());
            self.dom.clear();
            return;
        }

        self.loading = true;
        self.error_message = None;
        if matches!(intent, NavigateIntent::NewPage) {
            self.dom.clear();
        }

        self.pending_nav = Some((trimmed.clone(), intent));

        let (tx, rx) = mpsc::channel();
        self.fetch_rx = Some(rx);

        std::thread::spawn(move || {
            let outcome: FetchResult = (|| {
                let html = fetch_url(&trimmed).map_err(|e| e.to_string())?;
                Ok(parse_html(&html))
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

    fn apply_successful_navigation(&mut self, nodes: Vec<DomNode>) {
        let Some((url, intent)) = self.pending_nav.take() else {
            self.dom = nodes;
            return;
        };

        self.dom = nodes.clone();

        match intent {
            NavigateIntent::Reload => {
                if self.history.is_empty() {
                    self.history.push(HistoryEntry { url, nodes });
                    self.hist_index = 0;
                } else if self.hist_index < self.history.len() {
                    self.history[self.hist_index] = HistoryEntry { url, nodes };
                } else {
                    self.history.push(HistoryEntry { url, nodes });
                    self.hist_index = self.history.len() - 1;
                }
            }
            NavigateIntent::NewPage => {
                // Drop forward history, then append the new page (same tab).
                self.history.truncate(self.hist_index.saturating_add(1));
                self.history.push(HistoryEntry { url, nodes });
                self.hist_index = self.history.len().saturating_sub(1);
            }
        }
    }

    fn go_back(&mut self, ctx: &egui::Context) {
        if self.hist_index == 0 {
            return;
        }
        self.hist_index -= 1;
        let e = self.history[self.hist_index].clone();
        self.url_input = e.url;
        self.dom = e.nodes;
        self.error_message = None;
        self.sync_window_title(ctx);
    }

    fn go_forward(&mut self, ctx: &egui::Context) {
        if self.hist_index + 1 >= self.history.len() {
            return;
        }
        self.hist_index += 1;
        let e = self.history[self.hist_index].clone();
        self.url_input = e.url;
        self.dom = e.nodes;
        self.error_message = None;
        self.sync_window_title(ctx);
    }

    fn sync_window_title(&mut self, ctx: &egui::Context) {
        let doc_title = self
            .dom
            .iter()
            .find(|n| n.kind == DomNodeType::Title)
            .map(|n| n.text.as_str());

        let new_title = doc_title.unwrap_or(i18n::app_name(self.loc()));
        if new_title != self.window_title {
            self.window_title = new_title.to_string();
            ctx.send_viewport_cmd(ViewportCommand::Title(self.window_title.clone()));
        }
    }

    fn poll_fetch(&mut self, ctx: &egui::Context) {
        let Some(rx) = &self.fetch_rx else {
            return;
        };

        match rx.try_recv() {
            Ok(Ok(nodes)) => {
                self.loading = false;
                self.fetch_rx = None;
                self.apply_successful_navigation(nodes);
                self.sync_window_title(ctx);
                ctx.request_repaint();
            }
            Ok(Err(msg)) => {
                self.error_message = Some(msg);
                self.loading = false;
                self.fetch_rx = None;
                self.pending_nav = None;
                ctx.send_viewport_cmd(ViewportCommand::Title("Tonet".into()));
                self.window_title = "Tonet".to_string();
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Empty) => {
                ctx.request_repaint();
            }
            Err(mpsc::TryRecvError::Disconnected) => {
                self.loading = false;
                self.fetch_rx = None;
                self.pending_nav = None;
                self.error_message = Some(i18n::err_fetch_disconnected(self.loc()).to_string());
                ctx.request_repaint();
            }
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
        self.poll_fetch(ctx);
        self.poll_update_job(ctx);
        self.maybe_schedule_update_checks(ctx);

        let loc = self.loc();
        let can_back = self.hist_index > 0;
        let can_forward = self.hist_index + 1 < self.history.len();

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

                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(i18n::app_name(loc))
                            .strong()
                            .size(15.0)
                            .color(Color32::from_rgb(120, 175, 255)),
                    );
                    ui.separator();
                });

                let tb = show_chrome_toolbar(
                    ui,
                    loc,
                    &mut self.url_input,
                    self.loading,
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

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(err) = &self.error_message {
                show_error_panel(ui, loc, err);
                ui.add_space(8.0);
            }

            if self.loading {
                show_loading(ui, loc);
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if !self.loading && self.error_message.is_none() {
                        render_nodes(ui, loc, &self.dom);
                    } else if !self.loading && self.error_message.is_some() {
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
