//! Tonet online stub installer — Chrome-style minimal UI.
//!
//! Downloads the portable release package from GitHub/CDN, extracts locally, and launches Tonet.
//! Offline installs use Inno Setup (`installer/tonet.iss`) from CI.

#![cfg_attr(windows, windows_subsystem = "windows")]

mod branding;
mod install;
mod install_state;
mod i18n;
mod release;
mod version;
#[cfg(windows)]
mod uninstall;
#[cfg(windows)]
mod windows_util;

use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use eframe::egui::{self, Align, Color32, Layout, RichText, Sense, Vec2, ViewportCommand};
use i18n::Locale;
use install_state::{resolve_install_action, InstallAction};
use release::resolve_download_urls;

/// Overall progress 0..1 mapped from download + extract phases.
const DOWNLOAD_WEIGHT: f32 = 0.82;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InstallPhase {
    Preparing,
    Downloading,
    Updating,
    Installing,
    Done,
}

#[derive(Debug)]
enum WorkerMsg {
    Phase(InstallPhase),
    Progress(f32),
    FinishedOk {
        installed_exe: PathBuf,
        updated: bool,
    },
    FinishedAlreadyInstalled {
        version: String,
    },
    BeginUpdate {
        from_version: String,
        to_version: String,
    },
    FinishedErr(String),
}

fn send_progress(tx: &mpsc::Sender<WorkerMsg>, phase: InstallPhase, fraction: f32) {
    let _ = tx.send(WorkerMsg::Phase(phase));
    let _ = tx.send(WorkerMsg::Progress(fraction.clamp(0.0, 1.0)));
}

fn download_to_file(
    url: &str,
    dest: &std::path::Path,
    tx: &mpsc::Sender<WorkerMsg>,
) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("TonetSetup/2.0 (https://usetonet.com)")
        .build()
        .map_err(|e| e.to_string())?;
    let mut resp = client.get(url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("download failed: {}", resp.status()));
    }
    let total = resp.content_length();
    let mut f = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    let mut buf = [0u8; 64 * 1024];
    let mut got = 0u64;
    loop {
        let n = std::io::Read::read(&mut resp, &mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        f.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        got += n as u64;
        let frac = match total {
            Some(t) if t > 0 => (got as f32 / t as f32) * DOWNLOAD_WEIGHT,
            _ => (got as f32 / 50_000_000.0).min(DOWNLOAD_WEIGHT * 0.95),
        };
        send_progress(tx, InstallPhase::Downloading, frac);
    }
    send_progress(tx, InstallPhase::Downloading, DOWNLOAD_WEIGHT);
    Ok(())
}

fn download_with_cdn_fallback(
    primary_url: &str,
    cdn_url: Option<&str>,
    dest: &std::path::Path,
    tx: &mpsc::Sender<WorkerMsg>,
) -> Result<(), String> {
    match download_to_file(primary_url, dest, tx) {
        Ok(()) => Ok(()),
        Err(primary_err) => {
            let Some(fallback) = cdn_url.filter(|u| *u != primary_url) else {
                return Err(primary_err);
            };
            let _ = primary_err;
            download_to_file(fallback, dest, tx)
        }
    }
}

fn run_install_pipeline(tx: mpsc::Sender<WorkerMsg>) {
    send_progress(&tx, InstallPhase::Preparing, 0.02);

    let urls = match resolve_download_urls() {
        Ok(u) => u,
        Err(e) => {
            let _ = tx.send(WorkerMsg::FinishedErr(e.to_string()));
            return;
        }
    };

    let latest = urls.version.clone();
    let action = resolve_install_action(&latest);
    let was_update = matches!(&action, InstallAction::Update { .. });

    match action {
        InstallAction::AlreadyCurrent { installed_version } => {
            let _ = tx.send(WorkerMsg::FinishedAlreadyInstalled {
                version: installed_version,
            });
            return;
        }
        InstallAction::Update { from_version } => {
            let _ = tx.send(WorkerMsg::BeginUpdate {
                from_version,
                to_version: latest.clone(),
            });
        }
        InstallAction::Fresh => {}
    }

    let cdn_fallback = release::fetch_cdn_urls().ok();

    let tmp = std::env::temp_dir().join("tonet-setup");
    let _ = std::fs::create_dir_all(&tmp);

    #[cfg(target_os = "windows")]
    {
        let zip = match urls.windows_portable_zip {
            Some(z) => z,
            None => {
                let _ = tx.send(WorkerMsg::FinishedErr(
                    "No Windows portable package in the latest release.".into(),
                ));
                return;
            }
        };

        let zip_path = tmp.join("tonet-portable.zip");
        let cdn_zip = cdn_fallback
            .as_ref()
            .and_then(|c| c.windows_portable_zip.as_deref());
        if let Err(e) = download_with_cdn_fallback(&zip, cdn_zip, &zip_path, &tx) {
            let _ = tx.send(WorkerMsg::FinishedErr(e));
            return;
        }

        send_progress(&tx, InstallPhase::Installing, DOWNLOAD_WEIGHT);

        let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
        let dest = PathBuf::from(local).join("Programs").join("Tonet");

        let tx2 = tx.clone();
        let exe = match install::install_windows_portable_zip(&zip_path, &dest, |extract_frac| {
            let overall = DOWNLOAD_WEIGHT + extract_frac * (1.0 - DOWNLOAD_WEIGHT);
            send_progress(&tx2, InstallPhase::Installing, overall);
        }) {
            Ok(p) => p,
            Err(e) => {
                let _ = tx.send(WorkerMsg::FinishedErr(e.to_string()));
                return;
            }
        };

        if let Err(e) = install::finalize_windows_install(&dest, &exe, &urls.version) {
            let _ = tx.send(WorkerMsg::FinishedErr(e.to_string()));
            return;
        }

        send_progress(&tx, InstallPhase::Done, 1.0);
        let _ = tx.send(WorkerMsg::FinishedOk {
            installed_exe: exe,
            updated: was_update,
        });
    }

    #[cfg(target_os = "linux")]
    {
        let deb = match urls.linux_deb {
            Some(d) => d,
            None => {
                let _ = tx.send(WorkerMsg::FinishedErr(
                    "No Linux .deb in the latest release.".into(),
                ));
                return;
            }
        };
        let deb_path = tmp.join("tonet.deb");
        let cdn_deb = cdn_fallback.as_ref().and_then(|c| c.linux_deb.as_deref());
        if let Err(e) = download_with_cdn_fallback(&deb, cdn_deb, &deb_path, &tx) {
            let _ = tx.send(WorkerMsg::FinishedErr(e));
            return;
        }
        send_progress(&tx, InstallPhase::Installing, DOWNLOAD_WEIGHT);
        match install::install_linux_deb(&deb_path) {
            Ok(exe) => {
                if let Some(parent) = exe.parent() {
                    let _ = install_state::write_version_file(parent, &latest);
                }
                send_progress(&tx, InstallPhase::Done, 1.0);
                let _ = tx.send(WorkerMsg::FinishedOk {
                    installed_exe: exe,
                    updated: was_update,
                });
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::FinishedErr(e.to_string()));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let tgz = match urls.macos_tgz {
            Some(t) => t,
            None => {
                let _ = tx.send(WorkerMsg::FinishedErr(
                    "No macOS archive in the latest release.".into(),
                ));
                return;
            }
        };
        let path = tmp.join("tonet-macos.tar.gz");
        let cdn_tgz = cdn_fallback.as_ref().and_then(|c| c.macos_tgz.as_deref());
        if let Err(e) = download_with_cdn_fallback(&tgz, cdn_tgz, &path, &tx) {
            let _ = tx.send(WorkerMsg::FinishedErr(e));
            return;
        }
        send_progress(&tx, InstallPhase::Installing, DOWNLOAD_WEIGHT);
        match install::install_macos_tgz(&path) {
            Ok(exe) => {
                if let Some(parent) = exe.parent() {
                    let _ = install_state::write_version_file(parent, &latest);
                }
                send_progress(&tx, InstallPhase::Done, 1.0);
                let _ = tx.send(WorkerMsg::FinishedOk {
                    installed_exe: exe,
                    updated: was_update,
                });
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::FinishedErr(e.to_string()));
            }
        }
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        let _ = tx.send(WorkerMsg::FinishedErr(
            "Unsupported operating system.".into(),
        ));
    }
}

struct TonetSetupApp {
    loc: Locale,
    rx: mpsc::Receiver<WorkerMsg>,
    phase: InstallPhase,
    progress: f32,
    done_exe: Option<PathBuf>,
    error: Option<String>,
    info: Option<String>,
    updated: bool,
    launched: bool,
    dismiss_at: Option<std::time::Instant>,
    centered_once: bool,
}

impl TonetSetupApp {
    fn new(ctx: &egui::Context) -> Self {
        let (tx, rx) = mpsc::channel();
        let ctx2 = ctx.clone();
        std::thread::spawn(move || {
            run_install_pipeline(tx);
            ctx2.request_repaint();
        });
        Self {
            loc: Locale::detect(),
            rx,
            phase: InstallPhase::Preparing,
            progress: 0.0,
            done_exe: None,
            error: None,
            info: None,
            updated: false,
            launched: false,
            dismiss_at: None,
            centered_once: false,
        }
    }

    fn phase_label(&self) -> String {
        if let Some(info) = &self.info {
            return info.clone();
        }
        if self.error.is_some() {
            return self.error.clone().unwrap_or_default();
        }
        match self.phase {
            InstallPhase::Preparing => i18n::phase_preparing(self.loc).to_string(),
            InstallPhase::Downloading => i18n::phase_downloading(self.loc).to_string(),
            InstallPhase::Updating => i18n::phase_updating(self.loc).to_string(),
            InstallPhase::Installing => {
                if self.updated {
                    i18n::phase_installing_update(self.loc).to_string()
                } else {
                    i18n::phase_installing(self.loc).to_string()
                }
            }
            InstallPhase::Done => {
                if self.updated {
                    i18n::phase_done_updated(self.loc).to_string()
                } else {
                    i18n::phase_done(self.loc).to_string()
                }
            }
        }
    }
}

fn paint_window_controls(ui: &mut egui::Ui, ctx: &egui::Context) {
    let btn = |ui: &mut egui::Ui, label: &str| {
        let size = Vec2::new(36.0, 28.0);
        let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
        let hover = resp.hovered();
        if hover {
            ui.painter()
                .rect_filled(rect, 0.0, Color32::from_rgb(240, 242, 246));
        }
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(15.0),
            Color32::from_rgb(48, 52, 60),
        );
        resp
    };

    ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
        ui.add_space(4.0);
        if btn(ui, "✕").clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
        }
        if btn(ui, "−").clicked() {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
        }
    });
}

impl eframe::App for TonetSetupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if !self.centered_once {
            if let Some(cmd) = ViewportCommand::center_on_screen(ctx) {
                ctx.send_viewport_cmd(cmd);
                self.centered_once = true;
            }
        }

        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMsg::Phase(p) => self.phase = p,
                WorkerMsg::Progress(f) => self.progress = f,
                WorkerMsg::BeginUpdate {
                    from_version,
                    to_version,
                } => {
                    self.phase = InstallPhase::Updating;
                    self.updated = true;
                    self.progress = 0.05;
                    self.info = Some(i18n::phase_updating_detail(
                        self.loc,
                        &from_version,
                        &to_version,
                    ));
                }
                WorkerMsg::FinishedOk {
                    installed_exe,
                    updated,
                } => {
                    self.phase = InstallPhase::Done;
                    self.progress = 1.0;
                    self.updated = updated;
                    self.info = None;
                    self.done_exe = Some(installed_exe);
                }
                WorkerMsg::FinishedAlreadyInstalled { version } => {
                    self.phase = InstallPhase::Done;
                    self.progress = 1.0;
                    self.info = Some(i18n::already_installed(self.loc, &version));
                    self.dismiss_at = Some(std::time::Instant::now() + Duration::from_secs(4));
                }
                WorkerMsg::FinishedErr(e) => {
                    self.info = None;
                    self.error = Some(e);
                }
            }
        }

        if let Some(at) = self.dismiss_at {
            if self.done_exe.is_none() && self.info.is_some() && std::time::Instant::now() >= at {
                ctx.send_viewport_cmd(ViewportCommand::Close);
                return;
            }
        }

        if self.done_exe.is_some() && !self.launched {
            self.launched = true;
            if let Some(exe) = self.done_exe.clone() {
                let _ = std::process::Command::new(exe).spawn();
            }
            ctx.request_repaint_after(Duration::from_millis(400));
        }

        if self.launched && self.done_exe.is_some() {
            ctx.send_viewport_cmd(ViewportCommand::Close);
            return;
        }

        ctx.request_repaint_after(Duration::from_millis(50));

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::WHITE))
            .show(ctx, |ui| {
                paint_window_controls(ui, ctx);

                ui.vertical_centered(|ui| {
                    ui.add_space(36.0);
                    let text_color = if self.error.is_some() {
                        Color32::from_rgb(180, 48, 48)
                    } else if self.info.is_some() {
                        Color32::from_rgb(56, 88, 168)
                    } else {
                        branding::body_text()
                    };
                    ui.set_max_width(360.0);
                    ui.label(
                        RichText::new(self.phase_label())
                            .size(16.0)
                            .color(text_color),
                    );
                    ui.add_space(18.0);

                    let w = 280.0;
                    let h = 4.0;
                    let track = ui.allocate_response(Vec2::new(w, h), Sense::hover()).rect;
                    let painter = ui.painter_at(track);
                    painter.rect_filled(track, 2.0, branding::track());
                    if self.error.is_none() && self.info.is_none() {
                        let fill_w = (track.width() * self.progress).max(0.0);
                        if fill_w > 0.5 {
                            let fill = egui::Rect::from_min_size(
                                track.min,
                                Vec2::new(fill_w, track.height()),
                            );
                            painter.rect_filled(fill, 2.0, branding::accent());
                        }
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(14.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.add(
                            egui::Image::from_uri(branding::TONET_LOGO_URI)
                                .max_size(Vec2::splat(22.0)),
                        );
                        ui.label(
                            RichText::new(i18n::brand_wordmark(self.loc))
                                .size(18.0)
                                .color(branding::wordmark()),
                        );
                    });
                });
            });
    }
}

#[cfg(windows)]
fn is_uninstall_mode() -> bool {
    if std::env::args().any(|a| a == "--uninstall") {
        return true;
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().to_lowercase()))
        .is_some_and(|stem| stem.contains("uninstall"))
}

#[cfg(windows)]
fn run_uninstall_mode() {
    let dir = uninstall::default_install_dir();
    if let Err(e) = uninstall::run_uninstall(&dir) {
        uninstall::show_uninstall_error_message(&e.to_string());
    } else {
        uninstall::show_uninstall_finished_message();
    }
}

fn main() -> eframe::Result<()> {
    static RUSTLS_CRYPTO: std::sync::Once = std::sync::Once::new();
    RUSTLS_CRYPTO.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });

    #[cfg(windows)]
    if is_uninstall_mode() {
        run_uninstall_mode();
        return Ok(());
    }

    let loc = Locale::detect();
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(i18n::window_title(loc))
        .with_inner_size([500.0, 240.0])
        .with_resizable(false)
        .with_decorations(false)
        .with_maximize_button(false);
    if let Some(icon) = branding::window_icon() {
        viewport = viewport.with_icon(icon);
    }
    let native = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Tonet Setup",
        native,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            cc.egui_ctx.include_bytes(
                branding::TONET_LOGO_URI,
                egui::load::Bytes::Static(branding::TONET_SVG),
            );
            Ok(Box::new(TonetSetupApp::new(&cc.egui_ctx)))
        }),
    )
}
