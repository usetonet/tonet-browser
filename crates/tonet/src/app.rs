//! Tonet application: tabs, chrome, navigation history, locale, and update checks.

use std::cell::Cell;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ViewportCommand};

use crate::branding;
use crate::i18n::{self, Locale};
use crate::network::{fetch_favicon_from_candidates, fetch_url, guess_favicon_ext};
use crate::parser::{extract_favicon_candidates, parse_html};
use crate::renderer::render_nodes;
use crate::settings::{AppSettings, UpdatePolicy};
use crate::theme;
use crate::tab::{HistoryEntry, NavigateIntent, PageFetchData, Tab, DEFAULT_HOME_URL};
use crate::chrome::{show_chrome_toolbar, show_tab_bar};
use crate::ui::{
    show_error_panel, show_loading, show_settings_window, show_update_banner,
};
use crate::update;
use crate::window_chrome;
use crate::window_resize;

fn url_encode_query(query: &str) -> String {
    let mut encoded = String::with_capacity(query.len() * 3);
    for byte in query.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push('+'),
            _ => {
                use std::fmt::Write;
                let _ = write!(encoded, "%{byte:02X}");
            }
        }
    }
    encoded
}

fn duckduckgo_search_url(query: &str) -> String {
    format!("https://duckduckgo.com/?q={}", url_encode_query(query))
}

fn favicon_cache_uri(page_url: &str, ext: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    page_url.hash(&mut h);
    format!("bytes://favicon/{:016x}{ext}", h.finish())
}

/// Consume a key-down event only if it is NOT an OS auto-repeat.
/// Returns `true` on the first physical press and removes the event so it
/// won't fire again this frame.
fn consume_non_repeat(ctx: &egui::Context, key: egui::Key, need_command: bool) -> bool {
    ctx.input_mut(|input| {
        let pos = input.events.iter().position(|e| {
            matches!(e,
                egui::Event::Key {
                    key: k,
                    pressed: true,
                    repeat: false,
                    modifiers,
                    ..
                } if *k == key && (!need_command || modifiers.command)
            )
        });
        if let Some(idx) = pos {
            input.events.remove(idx);
            true
        } else {
            false
        }
    })
}

fn resolve_omnibox_input(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
    {
        return trimmed.to_string();
    }

    if trimmed.chars().any(|c| c.is_whitespace()) {
        return duckduckgo_search_url(trimmed);
    }

    let host_part = trimmed.split('/').next().unwrap_or(trimmed);

    if host_part.contains('.') {
        let segments: Vec<&str> = host_part.split('.').collect();
        if segments.len() >= 2 && segments.iter().all(|s| !s.is_empty()) {
            return format!("https://{trimmed}");
        }
    }

    if host_part.contains(':') {
        return format!("https://{trimmed}");
    }

    duckduckgo_search_url(trimmed)
}

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

    /// When set, next toolbar pass focuses the omnibox and selects all text (Ctrl/⌘+L).
    omnibox_focus_select_all: bool,

    /// Borderless window with in-app caption (Windows by default; see `window_chrome`).
    integrated_title_chrome: bool,

    /// Best-effort DWM corner rounding attempts (Windows integrated mode only).
    #[cfg_attr(not(windows), allow(dead_code))]
    dwm_corner_attempts: u8,

    /// True once an overflow browser instance has been spawned at MAX_TABS.
    /// Prevents spawning unlimited processes.
    max_tabs_overflow_spawned: bool,
}

impl TonetApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.include_bytes(
            branding::TONET_LOGO_URI,
            egui::load::Bytes::Static(branding::TONET_SVG),
        );

        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = theme::CHROME_BG;
        visuals.window_fill = theme::CONTENT_BG;
        visuals.widgets.noninteractive.bg_fill = theme::CONTENT_BG;
        visuals.extreme_bg_color = theme::OMNIBOX_FILL;
        visuals.faint_bg_color = theme::CHROME_BG;
        visuals.selection.bg_fill = Color32::from_rgb(55, 85, 135);
        visuals.selection.stroke = egui::Stroke::new(1.0, theme::ACCENT);
        visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, theme::ACCENT);
        let r = egui::Rounding::same(6.0);
        visuals.widgets.inactive.rounding = r;
        visuals.widgets.hovered.rounding = r;
        visuals.widgets.active.rounding = r;
        visuals.widgets.open.rounding = r;
        cc.egui_ctx.set_visuals(visuals);

        cc.egui_ctx.style_mut(|s| {
            s.spacing.button_padding = egui::vec2(6.0, 4.0);
            s.spacing.item_spacing = egui::vec2(6.0, 4.0);
            s.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::new(14.0, egui::FontFamily::Proportional),
            );
            s.text_styles.insert(
                egui::TextStyle::Small,
                egui::FontId::new(12.5, egui::FontFamily::Proportional),
            );
        });

        let settings = AppSettings::load();
        let integrated_title_chrome = window_chrome::integrated_title_chrome();
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
            omnibox_focus_select_all: false,
            integrated_title_chrome,
            dwm_corner_attempts: 0,
            max_tabs_overflow_spawned: false,
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

    const MAX_TABS: usize = 73;

    fn open_new_tab(&mut self, ctx: &egui::Context) {
        if self.tabs.len() >= Self::MAX_TABS {
            if !self.max_tabs_overflow_spawned {
                self.max_tabs_overflow_spawned = true;
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).spawn();
                }
            }
            return;
        }
        self.tabs.push(Tab::new(DEFAULT_HOME_URL));
        self.active_tab = self.tabs.len() - 1;
        self.sync_window_title(ctx);
    }

    fn close_tab_at(&mut self, index: usize, ctx: &egui::Context) {
        self.tabs[index].cancel_in_flight();
        if self.tabs.len() <= 1 {
            self.tabs[0] = Tab::new(DEFAULT_HOME_URL);
            self.active_tab = 0;
            self.sync_window_title(ctx);
            return;
        }
        self.tabs.remove(index);
        if index < self.active_tab {
            self.active_tab -= 1;
        } else if index == self.active_tab {
            self.active_tab = self.active_tab.min(self.tabs.len().saturating_sub(1));
        }
        self.sync_window_title(ctx);
    }

    fn start_fetch_with_intent(&mut self, intent: NavigateIntent) {
        let tab = self.active_tab_mut();
        let trimmed = tab.url_input.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        let was_new_tab = tab.show_new_tab;

        let resolved = resolve_omnibox_input(&trimmed);
        tab.url_input = resolved.clone();
        tab.show_new_tab = false;

        if was_new_tab && matches!(intent, NavigateIntent::NewPage) {
            tab.history.push(HistoryEntry {
                url: String::new(),
                nodes: Vec::new(),
            });
            tab.hist_index = 0;
        }

        tab.loading = true;
        tab.error_message = None;
        tab.favicon_uri.clear();
        tab.favicon_fetch_rx = None;
        if matches!(intent, NavigateIntent::NewPage) {
            tab.dom.clear();
        }

        tab.pending_nav = Some((resolved.clone(), intent));

        let (tx, rx) = mpsc::channel();
        tab.fetch_rx = Some(rx);

        let page_url = resolved;
        std::thread::spawn(move || {
            let outcome = (|| {
                let html = fetch_url(&page_url).map_err(|e| e.to_string())?;
                let nodes = parse_html(&html, &page_url);
                let favicon_candidates = extract_favicon_candidates(&html, &page_url);
                Ok(PageFetchData { nodes, favicon_candidates })
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
        tab.show_new_tab = e.url.is_empty() && e.nodes.is_empty();
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
        tab.show_new_tab = e.url.is_empty() && e.nodes.is_empty();
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
                Ok(Ok(data)) => {
                    tab.loading = false;
                    tab.fetch_rx = None;
                    let favicon_candidates = data.favicon_candidates;
                    tab.apply_successful_navigation(data.nodes);

                    if !favicon_candidates.is_empty() {
                        let (fav_tx, fav_rx) = mpsc::channel();
                        tab.favicon_fetch_rx = Some(fav_rx);
                        std::thread::spawn(move || {
                            let _ = fav_tx.send(fetch_favicon_from_candidates(&favicon_candidates));
                        });
                    }

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

    fn poll_favicons(&mut self, ctx: &egui::Context) {
        for tab in &mut self.tabs {
            let Some(rx) = &tab.favicon_fetch_rx else {
                continue;
            };
            match rx.try_recv() {
                Ok(Some(bytes)) => {
                    tab.favicon_fetch_rx = None;
                    let ext = guess_favicon_ext(&bytes);
                    let uri = favicon_cache_uri(&tab.url_input, ext);
                    ctx.include_bytes(
                        uri.clone(),
                        egui::load::Bytes::Shared(std::sync::Arc::from(bytes.as_slice())),
                    );
                    tab.favicon_uri = uri;
                    ctx.request_repaint();
                }
                Ok(None) => {
                    tab.favicon_fetch_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    tab.favicon_fetch_rx = None;
                }
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
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(not(windows))]
        let _ = frame;

        #[cfg(windows)]
        {
            if self.integrated_title_chrome && self.dwm_corner_attempts < 60 {
                if crate::platform_windows::try_apply_round_corners(frame) {
                    self.dwm_corner_attempts = 60;
                } else {
                    self.dwm_corner_attempts = self.dwm_corner_attempts.saturating_add(1);
                }
            }
        }

        window_resize::maybe_begin_native_resize(ctx, self.integrated_title_chrome);

        if let Some(url) = self.active_tab_mut().pending_link_navigation.take() {
            self.active_tab_mut().url_input = url;
            self.start_fetch_new();
        }

        self.poll_fetch(ctx);
        self.poll_favicons(ctx);
        self.poll_update_job(ctx);
        self.maybe_schedule_update_checks(ctx);

        if ctx.input(|i| {
            i.key_pressed(egui::Key::F5)
                || (i.modifiers.command && i.key_pressed(egui::Key::R))
        }) && !self.active_tab().loading
        {
            self.reload_page();
        }

        if consume_non_repeat(ctx, egui::Key::T, true) {
            self.open_new_tab(ctx);
        }
        if consume_non_repeat(ctx, egui::Key::W, true) {
            self.close_tab_at(self.active_tab, ctx);
        }

        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Tab)) {
            let n = self.tabs.len();
            if n > 1 {
                let next = if ctx.input(|i| i.modifiers.shift) {
                    if self.active_tab == 0 { n - 1 } else { self.active_tab - 1 }
                } else {
                    (self.active_tab + 1) % n
                };
                self.set_active_tab(next, ctx);
            }
            ctx.input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Key { key: egui::Key::Tab, .. })));
        }

        {
            let num_tabs = self.tabs.len();
            let digit_keys = [
                (egui::Key::Num1, 0usize),
                (egui::Key::Num2, 1),
                (egui::Key::Num3, 2),
                (egui::Key::Num4, 3),
                (egui::Key::Num5, 4),
                (egui::Key::Num6, 5),
                (egui::Key::Num7, 6),
                (egui::Key::Num8, 7),
            ];
            for (key, idx) in digit_keys {
                if ctx.input(|i| i.modifiers.command && i.key_pressed(key)) && idx < num_tabs {
                    self.set_active_tab(idx, ctx);
                }
            }
            if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Num9)) && num_tabs > 0
            {
                self.set_active_tab(num_tabs - 1, ctx);
            }
        }

        if !self.settings_open
            && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::L))
        {
            self.omnibox_focus_select_all = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.command && !i.modifiers.alt)
        {
            let omnibox_has_focus = ctx.memory(|m| m.has_focus(crate::ui::omnibox_id()));
            if !omnibox_has_focus {
                self.omnibox_focus_select_all = true;
                ctx.input_mut(|i| i.events.retain(|e| !matches!(e, egui::Event::Key { key: egui::Key::Tab, .. })));
            }
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
        let tab_favicons: Vec<String> = self
            .tabs
            .iter()
            .map(|t| {
                if t.show_new_tab {
                    branding::TONET_LOGO_URI.to_string()
                } else {
                    t.favicon_uri.clone()
                }
            })
            .collect();
        let can_close_tabs = true;

        egui::TopBottomPanel::top("tonet_top")
            .frame(
                egui::Frame::none()
                    .fill(theme::CHROME_BG)
                    .inner_margin(egui::Margin {
                        left: 0.0,
                        right: 0.0,
                        top: 0.0,
                        bottom: theme::SP,
                    }),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().item_spacing = egui::vec2(0.0, 0.0);

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
                        ui.add_space(theme::SP);
                    }
                }

                let tb_tabs = show_tab_bar(
                    ui,
                    ctx,
                    loc,
                    &tab_titles,
                    &tab_favicons,
                    self.active_tab,
                    can_close_tabs,
                    self.integrated_title_chrome,
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

                ui.add_space(1.0);
                let r = ui.available_rect_before_wrap();
                ui.painter().hline(
                    r.x_range(),
                    r.top(),
                    egui::Stroke::new(1.0, theme::SEPARATOR),
                );
                ui.add_space(1.0);

                let omnibox_focus = std::mem::take(&mut self.omnibox_focus_select_all);
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
                    omnibox_focus,
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
                if tb.stop_loading {
                    self.active_tab_mut().cancel_in_flight();
                }
                if tb.navigate {
                    self.start_fetch_new();
                }
                if tb.open_settings {
                    self.settings_open = true;
                }
            });

        let tab = self.active_tab_mut();
        let is_new_tab = tab.is_new_tab();
        egui::CentralPanel::default().show(ctx, |ui| {
            if is_new_tab {
                let nta = crate::new_tab::show_new_tab_page(
                    ui,
                    loc,
                    &mut tab.url_input,
                );
                if let Some(url) = nta.navigate_to {
                    tab.url_input = url;
                    tab.pending_link_navigation = Some(tab.url_input.clone());
                }
            } else {
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
                            render_nodes(
                                ui,
                                loc,
                                &tab.dom,
                                &mut tab.pending_link_navigation,
                            );
                        } else if !tab.loading && tab.error_message.is_some() {
                            ui.label(
                                egui::RichText::new(i18n::suggestion_fix_url(loc))
                                    .italics()
                                    .color(theme::LOADING_MUTED),
                            );
                        }
                    });
            }
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

        window_resize::update_resize_hover_cursor(ctx, self.integrated_title_chrome);
    }
}
