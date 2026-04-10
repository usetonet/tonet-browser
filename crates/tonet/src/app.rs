//! Aplicación principal: navegación, ajustes y comprobación de actualizaciones.

use std::cell::Cell;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ViewportCommand};

use crate::network::fetch_url;
use crate::parser::{parse_html, DomNode, DomNodeType};
use crate::renderer::render_nodes;
use crate::settings::{AppSettings, UpdatePolicy};
use crate::ui::{show_error_panel, show_loading, show_settings_window, show_top_bar, show_update_banner};
use crate::update;

type FetchResult = Result<Vec<DomNode>, String>;

/// Resultado de la comprobación de actualización (canal UI ↔ hilo).
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
        }
    }

    fn start_fetch(&mut self) {
        let trimmed = self.url_input.trim().to_string();
        if trimmed.is_empty() {
            self.error_message = Some("Escribe una URL válida (http o https).".to_string());
            self.dom.clear();
            return;
        }

        self.loading = true;
        self.error_message = None;
        self.dom.clear();

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

    fn sync_window_title(&mut self, ctx: &egui::Context) {
        let doc_title = self
            .dom
            .iter()
            .find(|n| n.kind == DomNodeType::Title)
            .map(|n| n.text.as_str());

        let new_title = doc_title.unwrap_or("Tonet");
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
                self.dom = nodes;
                self.loading = false;
                self.fetch_rx = None;
                self.sync_window_title(ctx);
                ctx.request_repaint();
            }
            Ok(Err(msg)) => {
                self.error_message = Some(msg);
                self.loading = false;
                self.fetch_rx = None;
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
                self.error_message = Some("La tarea de red finalizó de forma inesperada.".to_string());
                ctx.request_repaint();
            }
        }
    }

    fn spawn_update_check(&mut self) {
        if self.update_busy {
            return;
        }
        self.update_busy = true;
        self.update_status_line = "Consultando GitHub…".to_string();

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
                            format!("Estás al día (v{}).", env!("CARGO_PKG_VERSION"));
                    }
                    UpdateJobResult::UpdateAvailable { version } => {
                        self.update_status_line = format!(
                            "Hay una versión nueva: v{version}. Usa «Descargar» en el aviso o abre la página de releases."
                        );
                        if !self.update_banner_dismissed {
                            self.update_banner = Some(version);
                        }
                    }
                    UpdateJobResult::Error(e) => {
                        self.update_status_line = format!("No se pudo comprobar: {e}");
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
                self.update_status_line =
                    "La comprobación de actualizaciones se interrumpió.".to_string();
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

        egui::TopBottomPanel::top("tonet_top").show(ctx, |ui| {
            ui.add_space(6.0);
            ui.vertical(|ui| {
                if let Some(ver) = &self.update_banner {
                    if !self.update_banner_dismissed {
                        show_update_banner(
                            ui,
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
                        egui::RichText::new("Tonet")
                            .strong()
                            .size(15.0)
                            .color(Color32::from_rgb(120, 175, 255)),
                    );
                    ui.separator();
                    let bar = show_top_bar(ui, &mut self.url_input, self.loading);
                    if bar.navigate {
                        self.start_fetch();
                    }
                    if bar.open_settings {
                        self.settings_open = true;
                    }
                });
            });
            ui.add_space(4.0);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(err) = &self.error_message {
                show_error_panel(ui, err);
                ui.add_space(8.0);
            }

            if self.loading {
                show_loading(ui);
            }

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if !self.loading && self.error_message.is_none() {
                        render_nodes(ui, &self.dom);
                    } else if !self.loading && self.error_message.is_some() {
                        ui.label(
                            egui::RichText::new("Corrige la URL o prueba otra página ligera.")
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
