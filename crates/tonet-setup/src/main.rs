//! Tonet web installer: minimal UI, downloads latest GitHub release, installs Tonet.

mod install;
mod release;

use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, RichText, Sense, Stroke, Vec2};
use release::{fetch_latest_release, resolve_assets, GhAsset};

#[derive(Debug)]
enum WorkerMsg {
    Line(String),
    Progress { downloaded: u64, total: Option<u64> },
    FinishedOk { installed_exe: PathBuf },
    FinishedErr(String),
}

fn download_to_file(
    url: &str,
    dest: &std::path::Path,
    tx: &mpsc::Sender<WorkerMsg>,
) -> Result<(), String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("TonetSetup/1.0 (https://usetonet.com)")
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
        let _ = tx.send(WorkerMsg::Progress {
            downloaded: got,
            total,
        });
    }
    Ok(())
}

fn pick_windows_msi_zip(assets: &release::ResolvedAssets) -> (Option<GhAsset>, Option<GhAsset>) {
    (assets.windows_msi.clone(), assets.windows_portable_zip.clone())
}

fn run_install_pipeline(tx: mpsc::Sender<WorkerMsg>) {
    let send_line = |s: &str| {
        let _ = tx.send(WorkerMsg::Line(s.to_string()));
    };

    send_line("Motors turned on...");
    std::thread::sleep(Duration::from_millis(400));

    send_line("Connecting to internet...");
    let gh = match fetch_latest_release() {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(WorkerMsg::FinishedErr(e.to_string()));
            return;
        }
    };

    let assets = match resolve_assets(&gh) {
        Ok(a) => a,
        Err(e) => {
            let _ = tx.send(WorkerMsg::FinishedErr(e.to_string()));
            return;
        }
    };

    send_line("Waiting to download...");
    std::thread::sleep(Duration::from_millis(200));

    let tmp = std::env::temp_dir().join("tonet-setup");
    let _ = std::fs::create_dir_all(&tmp);

    #[cfg(target_os = "windows")]
    {
        let (msi, zip) = pick_windows_msi_zip(&assets);
        let msi_path = tmp.join("tonet-setup.msi");
        let zip_path = tmp.join("tonet-portable.zip");

        if let Some(ref a) = msi {
            send_line("Downloading...");
            if let Err(e) = download_to_file(&a.browser_download_url, &msi_path, &tx) {
                let _ = tx.send(WorkerMsg::FinishedErr(e));
                return;
            }
            send_line("Download complete. Please wait while the installer is verified.");
            std::thread::sleep(Duration::from_millis(500));
            send_line("Waiting to install");
            send_line("Installing...");
            match install::install_windows_msi(&msi_path) {
                Ok(()) => {
                    send_line("Installed");
                    let pf = PathBuf::from(r"C:\Program Files\Tonet\tonet.exe");
                    let exe = if pf.exists() {
                        pf
                    } else {
                        PathBuf::from(r"C:\Program Files (x86)\Tonet\tonet.exe")
                    };
                    let _ = install::create_desktop_shortcut(&exe);
                    let _ = tx.send(WorkerMsg::FinishedOk {
                        installed_exe: exe,
                    });
                    return;
                }
                Err(_) => {
                    let _ = tx.send(WorkerMsg::Line(
                        "MSI install unavailable; switching to per-user install…".into(),
                    ));
                }
            }
        }

        let z = match zip {
            Some(z) => z,
            None => {
                let _ = tx.send(WorkerMsg::FinishedErr(
                    "No Windows portable zip in the latest release.".into(),
                ));
                return;
            }
        };

        send_line("Downloading...");
        if let Err(e) = download_to_file(&z.browser_download_url, &zip_path, &tx) {
            let _ = tx.send(WorkerMsg::FinishedErr(e));
            return;
        }
        send_line("Download complete. Please wait while the installer is verified.");
        std::thread::sleep(Duration::from_millis(400));

        let local = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".into());
        let dest = PathBuf::from(local).join("Programs").join("Tonet");
        send_line("Installing...");
        let exe = match install::install_windows_portable_zip(&zip_path, &dest) {
            Ok(p) => p,
            Err(e) => {
                let _ = tx.send(WorkerMsg::FinishedErr(e.to_string()));
                return;
            }
        };
        if let Err(e) = install::create_desktop_shortcut(&exe) {
            let _ = tx.send(WorkerMsg::Line(format!("Shortcut: {e}")));
        }
        send_line("Installed");
        let _ = tx.send(WorkerMsg::FinishedOk { installed_exe: exe });
    }

    #[cfg(target_os = "linux")]
    {
        let deb = match assets.linux_deb.clone() {
            Some(d) => d,
            None => {
                let _ = tx.send(WorkerMsg::FinishedErr(
                    "No Linux .deb in the latest release.".into(),
                ));
                return;
            }
        };
        let deb_path = tmp.join("tonet.deb");
        send_line("Downloading...");
        if let Err(e) = download_to_file(&deb.browser_download_url, &deb_path, &tx) {
            let _ = tx.send(WorkerMsg::FinishedErr(e));
            return;
        }
        send_line("Download complete. Please wait while the installer is verified.");
        std::thread::sleep(Duration::from_millis(400));
        send_line("Installing...");
        match install::install_linux_deb(&deb_path) {
            Ok(exe) => {
                send_line("Installed");
                let _ = tx.send(WorkerMsg::FinishedOk { installed_exe: exe });
            }
            Err(e) => {
                let _ = tx.send(WorkerMsg::FinishedErr(e.to_string()));
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let tgz = match assets.macos_tgz.clone() {
            Some(t) => t,
            None => {
                let _ = tx.send(WorkerMsg::FinishedErr(
                    "No macOS archive in the latest release.".into(),
                ));
                return;
            }
        };
        let path = tmp.join("tonet-macos.tar.gz");
        send_line("Downloading...");
        if let Err(e) = download_to_file(&tgz.browser_download_url, &path, &tx) {
            let _ = tx.send(WorkerMsg::FinishedErr(e));
            return;
        }
        send_line("Download complete. Please wait while the installer is verified.");
        std::thread::sleep(Duration::from_millis(400));
        send_line("Installing...");
        match install::install_macos_tgz(&path) {
            Ok(exe) => {
                send_line("Installed");
                let _ = tx.send(WorkerMsg::FinishedOk { installed_exe: exe });
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
    rx: mpsc::Receiver<WorkerMsg>,
    line: String,
    downloaded: u64,
    total: Option<u64>,
    t0: Instant,
    bar_phase: f32,
    done_exe: Option<PathBuf>,
    error: Option<String>,
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
            rx,
            line: "On your marks...".into(),
            downloaded: 0,
            total: None,
            t0: Instant::now(),
            bar_phase: 0.0,
            done_exe: None,
            error: None,
        }
    }

    fn eta_seconds(&self) -> Option<u32> {
        let total = self.total?;
        if self.downloaded >= total {
            return Some(0);
        }
        let dt = Instant::now().duration_since(self.t0).as_secs_f64();
        if dt < 0.25 {
            return None;
        }
        let speed = self.downloaded as f64 / dt;
        if speed < 1.0 {
            return None;
        }
        let remain = (total.saturating_sub(self.downloaded)) as f64 / speed;
        Some(remain.ceil().max(0.0) as u32)
    }
}

impl eframe::App for TonetSetupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                WorkerMsg::Line(s) => {
                    if s == "Downloading..." {
                        self.t0 = Instant::now();
                    }
                    self.line = s;
                }
                WorkerMsg::Progress { downloaded, total } => {
                    self.downloaded = downloaded;
                    self.total = total;
                }
                WorkerMsg::FinishedOk { installed_exe } => {
                    self.line = "Installed".into();
                    self.done_exe = Some(installed_exe);
                }
                WorkerMsg::FinishedErr(e) => {
                    self.error = Some(e);
                }
            }
        }

        self.bar_phase += 0.08;
        if self.bar_phase > std::f32::consts::TAU {
            self.bar_phase -= std::f32::consts::TAU;
        }
        ctx.request_repaint_after(Duration::from_millis(32));

        let mut dl_line = self.line.clone();
        if self.line == "Downloading..." {
            if let Some(eta) = self.eta_seconds() {
                dl_line = format!("Downloading... {eta} second(s) remaining");
            }
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::WHITE))
            .show(ctx, |ui| {
                let rect = ui.max_rect();
                ui.painter().rect_stroke(
                    rect,
                    0.0,
                    Stroke::new(1.0, Color32::from_rgb(66, 133, 244)),
                );

                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.38);
                    let label = if let Some(ref e) = self.error {
                        RichText::new(e).color(Color32::from_rgb(180, 40, 40)).size(16.0)
                    } else {
                        RichText::new(&dl_line).color(Color32::BLACK).size(16.0)
                    };
                    ui.label(label);
                    ui.add_space(18.0);

                    let w = (ui.available_width() * 0.55).clamp(220.0, 520.0);
                    let h = 8.0;
                    let track = ui.allocate_response(Vec2::new(w, h), Sense::hover()).rect;
                    let painter = ui.painter_at(track);
                    painter.rect_filled(track, 2.0, Color32::from_gray(220));

                    let seg_w = (w * 0.18).max(36.0);
                    let travel = w - seg_w;
                    let x0 = track.left() + (travel * 0.5 * (1.0 + self.bar_phase.sin()));
                    let seg = egui::Rect::from_min_size(
                        egui::pos2(x0, track.top()),
                        Vec2::new(seg_w, h),
                    );
                    painter.rect_filled(seg, 2.0, Color32::from_rgb(66, 133, 244));

                    if self.done_exe.is_some() {
                        ui.add_space(22.0);
                        ui.label(
                            RichText::new("You can launch Tonet from your desktop shortcut.")
                                .size(14.0)
                                .color(Color32::from_gray(60)),
                        );
                    }
                });
            });
    }
}

fn main() -> eframe::Result<()> {
    static RUSTLS_CRYPTO: std::sync::Once = std::sync::Once::new();
    RUSTLS_CRYPTO.call_once(|| {
        let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    });

    let native = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Tonet Setup")
            .with_inner_size([520.0, 220.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Tonet Setup",
        native,
        Box::new(|cc| Ok(Box::new(TonetSetupApp::new(&cc.egui_ctx)))),
    )
}
