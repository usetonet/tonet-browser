//! Tonet application: tabs, chrome, navigation history, locale, and update checks.

use std::cell::Cell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ViewportCommand};

use crate::branding;
use crate::browser_log::BrowserLog;
use crate::chrome::{show_chrome_toolbar, show_tab_bar};
use crate::css_resolve::compute_dom_paint_hints;
use crate::i18n::{self, Locale};
use crate::internal_pages::{self, InternalRoute};
use crate::network::{
    fetch_favicon_from_candidates, fetch_stylesheets_from_urls, fetch_url, guess_favicon_ext,
};
use crate::parser::{extract_favicon_candidates, extract_stylesheet_candidates, parse_html};
use crate::renderer::render_nodes;
use crate::session_snapshot::SessionSnapshot;
use crate::settings::{AppSettings, SearchEngine, StartupPolicy, UiTheme, UpdatePolicy};
use crate::tab::{HistoryEntry, NavigateIntent, PageFetchData, Tab, DEFAULT_HOME_URL};
use crate::theme;
use crate::ui::{show_error_panel, show_loading, show_settings_window, show_update_banner};
use crate::update;
use crate::window_chrome;
use crate::window_resize;
use crate::css::{
    parse_stylesheet_bundle_rule_declarations, parse_stylesheet_bundle_to_rules,
    tokenize_stylesheet_bundle,
};

#[cfg(all(feature = "servo-engine", windows))]
fn servo_console_line_color(level: crate::tab::ServoConsoleLevel) -> Color32 {
    use crate::tab::ServoConsoleLevel as L;
    match level {
        L::Error => theme::error_title(),
        L::Warn => Color32::from_rgb(230, 178, 92),
        L::Info | L::Log => theme::accent(),
        L::Debug | L::Trace => theme::loading_muted(),
    }
}

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

fn search_url_for_query(engine: SearchEngine, query: &str) -> String {
    let q = url_encode_query(query);
    match engine {
        SearchEngine::Duckduckgo => format!("https://duckduckgo.com/?q={q}"),
        SearchEngine::Google => format!("https://www.google.com/search?q={q}"),
        SearchEngine::Brave => format!("https://search.brave.com/search?q={q}"),
    }
}

pub(crate) fn favicon_cache_uri(page_url: &str, ext: &str) -> String {
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

fn resolve_omnibox_input(input: &str, search_engine: SearchEngine) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed.starts_with("http://")
        || trimmed.starts_with("https://")
        || trimmed.starts_with("file://")
        || trimmed.starts_with("tonet://")
    {
        return trimmed.to_string();
    }

    if trimmed.chars().any(|c| c.is_whitespace()) {
        return search_url_for_query(search_engine, trimmed);
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

    search_url_for_query(search_engine, trimmed)
}

const MAX_TABS: usize = 73;

fn build_initial_tabs(settings: &AppSettings) -> (Vec<Tab>, usize) {
    match settings.startup_policy {
        StartupPolicy::NewTabPage => (vec![Tab::new(DEFAULT_HOME_URL)], 0),
        StartupPolicy::RestoreSession => SessionSnapshot::load()
            .and_then(|s| s.into_tabs(MAX_TABS))
            .unwrap_or_else(|| (vec![Tab::new(DEFAULT_HOME_URL)], 0)),
        StartupPolicy::OpenSpecificPages => {
            let urls: Vec<String> = settings
                .startup_urls
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(|l| resolve_omnibox_input(l, settings.search_engine))
                .filter(|u| !u.trim().is_empty())
                .take(MAX_TABS)
                .collect();
            if urls.is_empty() {
                (vec![Tab::new(DEFAULT_HOME_URL)], 0)
            } else {
                (urls.into_iter().map(Tab::new).collect(), 0)
            }
        }
    }
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

    browser_log: BrowserLog,
    internal_shortcuts_filter: String,
    confirm_reset_settings: bool,
    internal_hist_search: String,
    internal_dl_search: String,
    internal_hist_selected: HashSet<u64>,
    confirm_clear_history: bool,
    confirm_clear_downloads: bool,

    /// “Add shortcut” dialog state on the New Tab page.
    new_tab_add: crate::new_tab::NewTabAddState,

    /// First frame after launch: load active tab if it has a URL (restore / specific pages).
    pending_startup_fetch: bool,

    /// Native `pixels_per_point` from egui at startup (HiDPI baseline before user UI scale).
    integration_pixels_per_point: f32,

    /// Experimental Servo viewport (Windows popup when `servo-engine` + setting/env).
    servo_viewport: crate::servo_engine::ServoViewportRuntime,

    /// Central panel rect (egui points) when the active tab shows an `http(s)` page; drives Servo window placement.
    servo_content_rect: Option<egui::Rect>,

    /// Previous-frame [`Self::servo_content_rect`], used at the start of `update` to drop Servo keyboard capture
    /// on a chrome click before `forward_captured_keyboard` runs.
    #[cfg(all(feature = "servo-engine", windows))]
    servo_prev_content_rect: Option<egui::Rect>,
}

impl TonetApp {
    fn apply_egui_visuals(
        ctx: &egui::Context,
        settings: &AppSettings,
        integration_pixels_per_point: f32,
    ) {
        theme::set_active_ui_theme(settings.ui_theme);
        let ppp =
            (integration_pixels_per_point.max(0.01) * settings.clamped_ui_scale()).clamp(0.25, 8.0);
        ctx.set_pixels_per_point(ppp);
        let mut visuals = match settings.ui_theme {
            UiTheme::Dark => egui::Visuals::dark(),
            UiTheme::Light => egui::Visuals::light(),
        };
        visuals.panel_fill = theme::chrome_bg();
        visuals.window_fill = theme::content_bg();
        visuals.widgets.noninteractive.bg_fill = theme::content_bg();
        visuals.extreme_bg_color = theme::omnibox_fill();
        visuals.faint_bg_color = theme::chrome_bg();
        visuals.selection.bg_fill = match settings.ui_theme {
            UiTheme::Dark => Color32::from_rgb(55, 85, 135),
            UiTheme::Light => Color32::from_rgb(190, 215, 250),
        };
        visuals.selection.stroke = egui::Stroke::new(1.0, theme::accent());
        visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
        visuals.widgets.active.bg_stroke = egui::Stroke::new(1.5, theme::accent());
        let r = egui::Rounding::same(6.0);
        visuals.widgets.inactive.rounding = r;
        visuals.widgets.hovered.rounding = r;
        visuals.widgets.active.rounding = r;
        visuals.widgets.open.rounding = r;
        ctx.set_visuals(visuals);
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        cc.egui_ctx.include_bytes(
            branding::TONET_LOGO_URI,
            egui::load::Bytes::Static(branding::TONET_SVG),
        );

        let settings = AppSettings::load();
        crate::servo_engine::link_servo_when_enabled();
        let integration_pixels_per_point = cc.egui_ctx.pixels_per_point().max(0.01);
        Self::apply_egui_visuals(&cc.egui_ctx, &settings, integration_pixels_per_point);

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

        let integrated_title_chrome = window_chrome::integrated_title_chrome();
        let (tabs, active_tab) = build_initial_tabs(&settings);
        let mut this = Self {
            tabs,
            active_tab,
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
            browser_log: BrowserLog::load(),
            internal_shortcuts_filter: String::new(),
            confirm_reset_settings: false,
            internal_hist_search: String::new(),
            internal_dl_search: String::new(),
            internal_hist_selected: HashSet::new(),
            confirm_clear_history: false,
            confirm_clear_downloads: false,
            new_tab_add: crate::new_tab::NewTabAddState::default(),
            pending_startup_fetch: true,
            integration_pixels_per_point,
            servo_viewport: crate::servo_engine::ServoViewportRuntime::default(),
            servo_content_rect: None,
            #[cfg(all(feature = "servo-engine", windows))]
            servo_prev_content_rect: None,
        };
        this.sync_window_title(&cc.egui_ctx);
        this
    }

    fn persist_last_session(&self) {
        let urls: Vec<String> = self.tabs.iter().map(|t| t.url_input.clone()).collect();
        let snap = SessionSnapshot::from_app(self.active_tab, urls);
        let _ = snap.save();
    }

    fn ensure_http_tab_loaded(&mut self, ctx: &egui::Context) {
        #[cfg(all(feature = "servo-engine", windows))]
        let servo_viewport = self.settings.system.experimental_servo_viewport;
        let tab = self.active_tab_mut();
        let u = tab.url_input.trim();
        if tab.is_new_tab() {
            return;
        }
        #[cfg(all(feature = "servo-engine", windows))]
        {
            if crate::servo_engine::servo_supersedes_dom_paint(servo_viewport, u) {
                return;
            }
        }
        if internal_pages::parse_tonet_url(u).is_some() {
            return;
        }
        if !(u.starts_with("http://") || u.starts_with("https://")) {
            return;
        }
        if tab.loading || tab.fetch_rx.is_some() || !tab.dom.is_empty() {
            return;
        }
        self.start_fetch_new(ctx);
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
        if let Some(p) = internal_pages::parse_tonet_url(&tab.url_input) {
            return Self::clamp_strip_label(internal_pages::tab_title(p.route, loc), 24);
        }
        #[cfg(all(feature = "servo-engine", windows))]
        if let Some(ref t) = tab.servo_document_title {
            let t = t.trim();
            if !t.is_empty() {
                return Self::clamp_strip_label(t, 24);
            }
        }
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
        self.ensure_http_tab_loaded(ctx);
    }

    fn open_new_tab(&mut self, ctx: &egui::Context) {
        if self.tabs.len() >= MAX_TABS {
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

    /// New tab with a concrete URL (e.g. Servo context menu → “Open link in new Tonet tab”).
    #[cfg(all(feature = "servo-engine", windows))]
    fn open_new_tab_with_url(&mut self, ctx: &egui::Context, url: String) {
        if self.tabs.len() >= MAX_TABS {
            if !self.max_tabs_overflow_spawned {
                self.max_tabs_overflow_spawned = true;
                if let Ok(exe) = std::env::current_exe() {
                    let _ = std::process::Command::new(exe).spawn();
                }
            }
            return;
        }
        self.tabs.push(Tab::new(url));
        self.active_tab = self.tabs.len() - 1;
        self.sync_window_title(ctx);
        self.start_fetch_with_intent(NavigateIntent::NewPage, ctx);
    }

    fn close_tab_at(&mut self, index: usize, ctx: &egui::Context) {
        self.tabs[index].cancel_in_flight();
        if self.tabs.len() <= 1 {
            ctx.send_viewport_cmd(ViewportCommand::Close);
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

    fn start_fetch_with_intent(&mut self, intent: NavigateIntent, ctx: &egui::Context) {
        let search_engine = self.settings.search_engine;
        #[cfg(all(feature = "servo-engine", windows))]
        let servo_viewport = self.settings.system.experimental_servo_viewport;
        let tab = self.active_tab_mut();
        let trimmed = tab.url_input.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        #[cfg(all(feature = "servo-engine", windows))]
        let tonet_via_servo = internal_pages::parse_tonet_url(&trimmed).is_some()
            && crate::servo_engine::servo_supersedes_dom_paint(servo_viewport, trimmed.as_str());
        #[cfg(not(all(feature = "servo-engine", windows)))]
        let tonet_via_servo = false;

        if let Some(parsed) = internal_pages::parse_tonet_url(&trimmed) {
            if !tonet_via_servo {
                let canonical = parsed.normalized_url();
                let was_new_tab = tab.show_new_tab;
                tab.cancel_in_flight();
                tab.show_new_tab = false;
                tab.loading = false;
                tab.error_message = None;
                tab.fetch_rx = None;
                tab.favicon_fetch_rx = None;
                tab.favicon_uri.clear();
                tab.dom.clear();
                tab.stylesheet_urls.clear();
                tab.url_input = canonical.clone();
                if was_new_tab && matches!(intent, NavigateIntent::NewPage) {
                    tab.history.push(HistoryEntry {
                        url: String::new(),
                        nodes: Vec::new(),
                        stylesheet_urls: Vec::new(),
                    });
                    tab.hist_index = 0;
                }
                tab.pending_nav = Some((canonical.clone(), intent));
                tab.apply_successful_navigation(Vec::new());
                self.sync_window_title(ctx);
                return;
            }
        }

        let was_new_tab = tab.show_new_tab;

        let mut resolved = resolve_omnibox_input(&trimmed, search_engine);
        if let Some(p) = internal_pages::parse_tonet_url(resolved.trim()) {
            resolved = p.normalized_url();
        }
        tab.url_input = resolved.clone();
        tab.show_new_tab = false;

        if was_new_tab && matches!(intent, NavigateIntent::NewPage) {
            tab.history.push(HistoryEntry {
                url: String::new(),
                nodes: Vec::new(),
                stylesheet_urls: Vec::new(),
            });
            tab.hist_index = 0;
        }

        #[cfg(all(feature = "servo-engine", windows))]
        {
            if crate::servo_engine::servo_supersedes_dom_paint(servo_viewport, tab.url_input.trim()) {
                tab.cancel_in_flight();
                tab.loading = true;
                tab.error_message = None;
                tab.favicon_uri.clear();
                tab.favicon_fetch_rx = None;
                tab.stylesheet_fetch_rx = None;
                tab.stylesheet_urls.clear();
                tab.loaded_stylesheets.clear();
                tab.loaded_stylesheet_tokens.clear();
                tab.loaded_stylesheet_rules.clear();
                tab.loaded_stylesheet_parsed.clear();
                if matches!(intent, NavigateIntent::NewPage) {
                    tab.dom.clear();
                }
                tab.pending_nav = Some((resolved.clone(), intent));
                tab.apply_successful_navigation(Vec::new());
                tab.fetch_rx = None;
                if matches!(intent, NavigateIntent::Reload) {
                    self.servo_viewport.webview_reload();
                }
                self.sync_window_title(ctx);
                ctx.request_repaint();
                return;
            }
        }

        tab.loading = true;
        tab.error_message = None;
        tab.favicon_uri.clear();
        tab.favicon_fetch_rx = None;
        tab.stylesheet_fetch_rx = None;
        tab.stylesheet_urls.clear();
        tab.loaded_stylesheets.clear();
        tab.loaded_stylesheet_tokens.clear();
        tab.loaded_stylesheet_rules.clear();
        tab.loaded_stylesheet_parsed.clear();
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
                let raw_html = html.clone();
                let nodes = parse_html(&html, &page_url);
                let favicon_candidates = extract_favicon_candidates(&html, &page_url);
                let stylesheet_candidates = extract_stylesheet_candidates(&html, &page_url);
                Ok(PageFetchData {
                    nodes,
                    favicon_candidates,
                    stylesheet_candidates,
                    raw_html,
                })
            })();
            let _ = tx.send(outcome);
        });
    }

    fn start_fetch_new(&mut self, ctx: &egui::Context) {
        self.start_fetch_with_intent(NavigateIntent::NewPage, ctx);
    }

    fn reload_page(&mut self, ctx: &egui::Context) {
        ctx.memory_mut(|m| m.surrender_focus(crate::ui::omnibox_id()));
        #[cfg(all(feature = "servo-engine", windows))]
        {
            let t = self.active_tab().url_input.trim();
            if crate::servo_engine::servo_supersedes_dom_paint(self.settings.system.experimental_servo_viewport, t)
            {
                self.active_tab_mut().loading = true;
                self.servo_viewport.webview_reload();
                self.sync_window_title(ctx);
                ctx.request_repaint();
                return;
            }
        }
        self.start_fetch_with_intent(NavigateIntent::Reload, ctx);
    }

    fn go_back(&mut self, ctx: &egui::Context) {
        ctx.memory_mut(|m| m.surrender_focus(crate::ui::omnibox_id()));
        #[cfg(all(feature = "servo-engine", windows))]
        {
            let t = self.active_tab().url_input.trim();
            if crate::servo_engine::servo_supersedes_dom_paint(self.settings.system.experimental_servo_viewport, t)
            {
                if self.servo_viewport.webview_go_back() {
                    self.sync_window_title(ctx);
                    ctx.request_repaint();
                }
                return;
            }
        }
        let tab = self.active_tab_mut();
        if tab.hist_index == 0 {
            return;
        }
        tab.hist_index -= 1;
        let e = tab.history[tab.hist_index].clone();
        tab.show_new_tab = e.url.is_empty() && e.nodes.is_empty();
        tab.url_input = e.url;
        tab.dom = e.nodes;
        tab.stylesheet_urls = e.stylesheet_urls;
        tab.stylesheet_fetch_rx = None;
        tab.loaded_stylesheets.clear();
        tab.loaded_stylesheet_tokens.clear();
        tab.loaded_stylesheet_rules.clear();
        tab.loaded_stylesheet_parsed.clear();
        tab.error_message = None;
        self.sync_window_title(ctx);
    }

    fn go_forward(&mut self, ctx: &egui::Context) {
        ctx.memory_mut(|m| m.surrender_focus(crate::ui::omnibox_id()));
        #[cfg(all(feature = "servo-engine", windows))]
        {
            let t = self.active_tab().url_input.trim();
            if crate::servo_engine::servo_supersedes_dom_paint(self.settings.system.experimental_servo_viewport, t)
            {
                if self.servo_viewport.webview_go_forward() {
                    self.sync_window_title(ctx);
                    ctx.request_repaint();
                }
                return;
            }
        }
        let tab = self.active_tab_mut();
        if tab.hist_index + 1 >= tab.history.len() {
            return;
        }
        tab.hist_index += 1;
        let e = tab.history[tab.hist_index].clone();
        tab.show_new_tab = e.url.is_empty() && e.nodes.is_empty();
        tab.url_input = e.url;
        tab.dom = e.nodes;
        tab.stylesheet_urls = e.stylesheet_urls;
        tab.stylesheet_fetch_rx = None;
        tab.loaded_stylesheets.clear();
        tab.loaded_stylesheet_tokens.clear();
        tab.loaded_stylesheet_rules.clear();
        tab.loaded_stylesheet_parsed.clear();
        tab.error_message = None;
        self.sync_window_title(ctx);
    }

    fn sync_window_title(&mut self, ctx: &egui::Context) {
        let tab = self.active_tab();
        let loc = self.loc();
        let servo_win_title = {
            #[cfg(all(feature = "servo-engine", windows))]
            {
                tab.servo_document_title.as_ref().and_then(|t| {
                    let t = t.trim();
                    (!t.is_empty()).then(|| t.to_string())
                })
            }
            #[cfg(not(all(feature = "servo-engine", windows)))]
            {
                None::<String>
            }
        };
        let new_title: String =
            if let Some(p) = internal_pages::parse_tonet_url(tab.url_input.trim()) {
                internal_pages::tab_title(p.route, loc).to_string()
            } else if let Some(t) = servo_win_title {
                t
            } else if let Some(t) = tab.doc_title_trimmed() {
                t.to_string()
            } else {
                i18n::app_name(loc).to_string()
            };
        if new_title != self.window_title {
            self.window_title = new_title;
            ctx.send_viewport_cmd(ViewportCommand::Title(self.window_title.clone()));
        }
    }

    fn poll_fetch(&mut self, ctx: &egui::Context) {
        let loc = self.loc();
        let active = self.active_tab;
        let mut title_dirty = false;
        let mut reset_window_title = false;
        let mut visit_queue: Vec<(String, Option<String>, Option<String>)> = Vec::new();

        for (i, tab) in self.tabs.iter_mut().enumerate() {
            let Some(rx) = &tab.fetch_rx else {
                continue;
            };

            match rx.try_recv() {
                Ok(Ok(data)) => {
                    tab.loading = false;
                    tab.fetch_rx = None;
                    let favicon_candidates = data.favicon_candidates;
                    tab.stylesheet_urls = data.stylesheet_candidates;
                    tab.apply_successful_navigation(data.nodes);

                    let page_url = tab.url_input.trim().to_string();
                    if page_url.starts_with("http://") || page_url.starts_with("https://") {
                        let title = tab.doc_title_trimmed().map(|s| s.to_string());
                        let saved_path = self
                            .settings
                            .resolved_download_directory()
                            .and_then(|root| {
                                crate::browser_log::save_page_html_snapshot(
                                    &root,
                                    &page_url,
                                    &data.raw_html,
                                )
                            })
                            .map(|p| p.display().to_string());
                        visit_queue.push((page_url, title, saved_path));
                    }

                    if !favicon_candidates.is_empty() {
                        let (fav_tx, fav_rx) = mpsc::channel();
                        tab.favicon_fetch_rx = Some(fav_rx);
                        std::thread::spawn(move || {
                            let _ = fav_tx.send(fetch_favicon_from_candidates(&favicon_candidates));
                        });
                    }

                    tab.stylesheet_fetch_rx = None;
                    if !tab.stylesheet_urls.is_empty() {
                        let urls = tab.stylesheet_urls.clone();
                        tab.loaded_stylesheets.clear();
                        tab.loaded_stylesheet_tokens.clear();
                        tab.loaded_stylesheet_rules.clear();
                        tab.loaded_stylesheet_parsed.clear();
                        let (sheet_tx, sheet_rx) = mpsc::channel();
                        tab.stylesheet_fetch_rx = Some(sheet_rx);
                        std::thread::spawn(move || {
                            let _ = sheet_tx.send(fetch_stylesheets_from_urls(&urls));
                        });
                    } else {
                        tab.loaded_stylesheets.clear();
                        tab.loaded_stylesheet_tokens.clear();
                        tab.loaded_stylesheet_rules.clear();
                        tab.loaded_stylesheet_parsed.clear();
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
                    tab.stylesheet_fetch_rx = None;
                    tab.stylesheet_urls.clear();
                    tab.loaded_stylesheets.clear();
                    tab.loaded_stylesheet_tokens.clear();
                    tab.loaded_stylesheet_rules.clear();
                    tab.loaded_stylesheet_parsed.clear();
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
                    tab.stylesheet_fetch_rx = None;
                    tab.stylesheet_urls.clear();
                    tab.loaded_stylesheets.clear();
                    tab.loaded_stylesheet_tokens.clear();
                    tab.loaded_stylesheet_rules.clear();
                    tab.loaded_stylesheet_parsed.clear();
                    tab.error_message = Some(i18n::err_fetch_disconnected(loc).to_string());
                    ctx.request_repaint();
                }
            }
        }

        for (page_url, title, saved_path) in visit_queue {
            self.browser_log
                .record_visit(page_url.clone(), title.clone());
            self.browser_log
                .record_page_fetch(&page_url, title, saved_path);
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

    fn poll_stylesheets(&mut self, ctx: &egui::Context) {
        for tab in &mut self.tabs {
            let Some(rx) = &tab.stylesheet_fetch_rx else {
                continue;
            };
            match rx.try_recv() {
                Ok(v) => {
                    tab.stylesheet_fetch_rx = None;
                    let tok = tokenize_stylesheet_bundle(&v);
                    tab.loaded_stylesheet_rules = parse_stylesheet_bundle_to_rules(&tok);
                    tab.loaded_stylesheet_parsed =
                        parse_stylesheet_bundle_rule_declarations(&tab.loaded_stylesheet_rules);
                    tab.loaded_stylesheet_tokens = tok;
                    tab.loaded_stylesheets = v;
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint();
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    tab.stylesheet_fetch_rx = None;
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
        Self::apply_egui_visuals(ctx, &self.settings, self.integration_pixels_per_point);

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

        if self.pending_startup_fetch {
            self.pending_startup_fetch = false;
            let u = self.active_tab().url_input.trim();
            if !u.is_empty() && !self.active_tab().is_new_tab() {
                self.start_fetch_new(ctx);
            }
        }

        if let Some(url) = self.active_tab_mut().pending_link_navigation.take() {
            self.active_tab_mut().url_input = url;
            self.start_fetch_new(ctx);
        }

        self.poll_fetch(ctx);
        self.poll_favicons(ctx);
        self.poll_stylesheets(ctx);
        self.poll_update_job(ctx);
        self.maybe_schedule_update_checks(ctx);

        #[cfg(all(feature = "servo-engine", windows))]
        {
            self.servo_viewport
                .release_servo_keyboard_capture(ctx, self.servo_prev_content_rect);
            let tab_url = self.active_tab().url_input.clone();
            self.servo_viewport.forward_captured_keyboard(
                ctx,
                self.settings.system.experimental_servo_viewport,
                tab_url.as_str(),
            );
        }

        if ctx.input(|i| {
            i.key_pressed(egui::Key::F5) || (i.modifiers.command && i.key_pressed(egui::Key::R))
        }) && !self.active_tab().loading
        {
            self.reload_page(ctx);
        }

        if consume_non_repeat(ctx, egui::Key::T, true) {
            self.open_new_tab(ctx);
        }
        if consume_non_repeat(ctx, egui::Key::W, true) {
            self.close_tab_at(self.active_tab, ctx);
        }

        if consume_non_repeat(ctx, egui::Key::H, true) {
            self.active_tab_mut().url_input = InternalRoute::History.canonical_url().to_string();
            self.start_fetch_new(ctx);
        }
        if consume_non_repeat(ctx, egui::Key::J, true) {
            self.active_tab_mut().url_input = InternalRoute::Downloads.canonical_url().to_string();
            self.start_fetch_new(ctx);
        }

        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Tab)) {
            let n = self.tabs.len();
            if n > 1 {
                let next = if ctx.input(|i| i.modifiers.shift) {
                    if self.active_tab == 0 {
                        n - 1
                    } else {
                        self.active_tab - 1
                    }
                } else {
                    (self.active_tab + 1) % n
                };
                self.set_active_tab(next, ctx);
            }
            ctx.input_mut(|i| {
                i.events.retain(|e| {
                    !matches!(
                        e,
                        egui::Event::Key {
                            key: egui::Key::Tab,
                            ..
                        }
                    )
                })
            });
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

        if !self.settings_open && ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::L))
        {
            self.omnibox_focus_select_all = true;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Tab) && !i.modifiers.command && !i.modifiers.alt)
        {
            let omnibox_has_focus = ctx.memory(|m| m.has_focus(crate::ui::omnibox_id()));
            if !omnibox_has_focus {
                self.omnibox_focus_select_all = true;
                ctx.input_mut(|i| {
                    i.events.retain(|e| {
                        !matches!(
                            e,
                            egui::Event::Key {
                                key: egui::Key::Tab,
                                ..
                            }
                        )
                    })
                });
            }
        }

        let loc = self.loc();
        let tab = self.active_tab();
        #[cfg(all(feature = "servo-engine", windows))]
        let (can_back, can_forward) = {
            let mut b = tab.hist_index > 0;
            let mut f = tab.hist_index + 1 < tab.history.len();
            let servo_viewport = self.settings.system.experimental_servo_viewport;
            if crate::servo_engine::servo_supersedes_dom_paint(servo_viewport, tab.url_input.trim()) {
                if let Some((bb, ff)) = tab.servo_chrome_nav {
                    b = bb;
                    f = ff;
                }
            }
            (b, f)
        };
        #[cfg(not(all(feature = "servo-engine", windows)))]
        let (can_back, can_forward) = (tab.hist_index > 0, tab.hist_index + 1 < tab.history.len());
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
                } else if internal_pages::parse_tonet_url(t.url_input.trim()).is_some() {
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
                    .fill(theme::chrome_bg())
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

                #[cfg(all(feature = "servo-engine", windows))]
                if crate::servo_engine::viewport_runtime_requested(
                    self.settings.system.experimental_servo_viewport,
                ) {
                    self.servo_viewport
                        .show_web_notification_toast(ctx, loc);
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
                    egui::Stroke::new(1.0, theme::separator()),
                );
                ui.add_space(1.0);

                let omnibox_history_suggestions =
                    crate::browser_log::history_suggestions_for_omnibox(
                        &self.browser_log.visits,
                        self.active_tab().url_input.trim(),
                        8,
                    );
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
                    omnibox_history_suggestions.as_slice(),
                );
                if tb.go_back {
                    self.go_back(ctx);
                }
                if tb.go_forward {
                    self.go_forward(ctx);
                }
                if tb.reload {
                    self.reload_page(ctx);
                }
                if tb.stop_loading {
                    #[cfg(all(feature = "servo-engine", windows))]
                    let servo_supersedes = crate::servo_engine::servo_supersedes_dom_paint(
                        self.settings.system.experimental_servo_viewport,
                        self.active_tab().url_input.trim(),
                    );
                    #[cfg(not(all(feature = "servo-engine", windows)))]
                    let servo_supersedes = false;

                    // Servo’s public `WebView` API has no documented “stop load”; avoid
                    // `cancel_in_flight` here so we do not wipe shell state synced from Servo.
                    if !servo_supersedes {
                        self.active_tab_mut().cancel_in_flight();
                    }
                }
                if tb.navigate {
                    ctx.memory_mut(|m| m.surrender_focus(crate::ui::omnibox_id()));
                    self.start_fetch_new(ctx);
                }
                if tb.navigate_to_settings {
                    ctx.memory_mut(|m| m.surrender_focus(crate::ui::omnibox_id()));
                    self.active_tab_mut().url_input =
                        InternalRoute::Settings.canonical_url().to_string();
                    self.start_fetch_new(ctx);
                }
                if tb.open_settings {
                    self.settings_open = true;
                }
            });

        let current = env!("CARGO_PKG_VERSION");
        let active_i = self.active_tab;
        let is_new_tab = self.tabs[active_i].is_new_tab();
        let internal_route = if !is_new_tab {
            let u = self.tabs[active_i].url_input.trim();
            match internal_pages::parse_tonet_url(u) {
                Some(p) => {
                    #[cfg(all(feature = "servo-engine", windows))]
                    {
                        if crate::servo_engine::servo_supersedes_dom_paint(
                            self.settings.system.experimental_servo_viewport,
                            u,
                        ) {
                            None
                        } else {
                            Some(p)
                        }
                    }
                    #[cfg(not(all(feature = "servo-engine", windows)))]
                    {
                        Some(p)
                    }
                }
                None => None,
            }
        } else {
            None
        };
        let check_now = Rc::new(Cell::new(false));
        let check_now_settings = check_now.clone();

        egui::CentralPanel::default().show(ctx, |ui| {
            self.servo_content_rect = None;
            let tab = &mut self.tabs[active_i];
            if is_new_tab {
                let nta = crate::new_tab::show_new_tab_page(
                    ui,
                    ctx,
                    loc,
                    &mut tab.url_input,
                    &mut self.settings,
                    &mut self.new_tab_add,
                );
                if let Some(url) = nta.navigate_to {
                    tab.url_input = url;
                    tab.pending_link_navigation = Some(tab.url_input.clone());
                }
                if nta.need_save {
                    let _ = self.settings.save();
                }
            } else if let Some(parsed) = internal_route {
                let route = parsed.route;
                let settings_url = tab.url_input.clone();
                let out = match route {
                    InternalRoute::Settings => internal_pages::show_settings_page(
                        ui,
                        loc,
                        route,
                        settings_url.as_str(),
                        &mut self.settings,
                        self.update_busy,
                        &self.update_status_line,
                        current,
                        &mut self.internal_shortcuts_filter,
                        &mut self.confirm_reset_settings,
                        |s| {
                            let _ = s.save();
                        },
                        || {
                            check_now_settings.set(true);
                        },
                    ),
                    InternalRoute::Downloads => internal_pages::show_downloads_page(
                        ui,
                        ctx,
                        loc,
                        route,
                        &mut self.browser_log,
                        &mut self.internal_dl_search,
                        &mut self.confirm_clear_downloads,
                    ),
                    InternalRoute::History => internal_pages::show_history_page(
                        ui,
                        ctx,
                        loc,
                        route,
                        &mut self.browser_log,
                        &mut self.internal_hist_search,
                        &mut self.internal_hist_selected,
                        &mut self.confirm_clear_history,
                    ),
                };
                if let Some(url) = out.navigate_to {
                    tab.pending_link_navigation = Some(url);
                }
                #[cfg(all(feature = "servo-engine", windows))]
                if out.clear_servo_site_permissions {
                    crate::servo_engine::permission_store::remove_file();
                    self.servo_viewport.clear_servo_permission_memory();
                }
            } else {
                let url_trim = tab.url_input.trim();
                #[cfg(all(feature = "servo-engine", windows))]
                let tonet_servo_paint = internal_pages::parse_tonet_url(url_trim).is_some()
                    && crate::servo_engine::servo_supersedes_dom_paint(
                        self.settings.system.experimental_servo_viewport,
                        url_trim,
                    );
                #[cfg(not(all(feature = "servo-engine", windows)))]
                let tonet_servo_paint = false;
                if url_trim.starts_with("http://")
                    || url_trim.starts_with("https://")
                    || tonet_servo_paint
                {
                    self.servo_content_rect = Some(ui.max_rect());
                }
                if let Some(err) = &tab.error_message {
                    show_error_panel(ui, loc, err);
                    ui.add_space(8.0);
                }

                if tab.loading {
                    show_loading(ui, loc);
                }

                #[cfg(feature = "servo-engine")]
                {
                    let http = url_trim.starts_with("http://") || url_trim.starts_with("https://");
                    #[cfg(all(feature = "servo-engine", windows))]
                    if http && !crate::servo_engine::servo_supersedes_dom_paint(false, tab.url_input.trim())
                    {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(
                                egui::RichText::new(i18n::servo_windows_engine_disabled_hint(loc))
                                    .small()
                                    .color(theme::loading_muted()),
                            );
                        });
                        ui.add_space(6.0);
                    }
                    #[cfg(all(feature = "servo-engine", not(windows)))]
                    {
                        let env_on = std::env::var_os("TONET_SERVO_VIEWPORT")
                            .as_deref()
                            .is_some_and(|v| v == "1");
                        if http && !self.settings.system.experimental_servo_viewport && !env_on {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    egui::RichText::new(i18n::servo_compiled_activate_hint(loc))
                                        .small()
                                        .color(theme::loading_muted()),
                                );
                            });
                            ui.add_space(6.0);
                        }
                    }
                }

                let servo_dom = crate::servo_engine::servo_supersedes_dom_paint(
                    self.settings.system.experimental_servo_viewport,
                    tab.url_input.as_str(),
                ) && !tab.loading
                    && tab.error_message.is_none();

                if servo_dom {
                    // Leave a right gutter for a visible scrollbar strip; Win32 popup matches `inner` only.
                    const SERVO_SCROLL_GUTTER: f32 = 14.0;
                    #[cfg(all(feature = "servo-engine", windows))]
                    const SERVO_CONSOLE_STRIP_H: f32 = 120.0;
                    let full = ui.max_rect();
                    #[cfg(all(feature = "servo-engine", windows))]
                    let console_h = if tab.servo_console.is_empty() {
                        0.0
                    } else {
                        SERVO_CONSOLE_STRIP_H
                    };
                    #[cfg(not(all(feature = "servo-engine", windows)))]
                    let console_h = 0.0_f32;
                    let inner = egui::Rect::from_min_max(
                        full.min,
                        full.max - egui::vec2(SERVO_SCROLL_GUTTER, console_h),
                    );
                    self.servo_content_rect = Some(inner);
                    ui.allocate_rect(inner, egui::Sense::click_and_drag());
                    #[cfg(all(feature = "servo-engine", windows))]
                    {
                        let ppp = ui.ctx().pixels_per_point();
                        self.servo_viewport
                            .feed_servo_slint_egui_pointer(ctx, inner, ppp);
                        self.servo_viewport
                            .paint_servo_slint_embed_in_rect(ctx, ui, inner);
                    }
                    let track = egui::Rect::from_min_max(
                        egui::pos2(inner.right(), inner.top()),
                        egui::pos2(full.max.x, inner.max.y),
                    );
                    let p = ui.painter();
                    p.rect_filled(track, 3.0, theme::servo_scroll_gutter_fill());
                    p.rect_stroke(
                        track,
                        3.0,
                        egui::Stroke::new(1.0, theme::separator()),
                    );
                    let cx = track.center().x;
                    p.line_segment(
                        [
                            egui::pos2(cx, track.top() + 5.0),
                            egui::pos2(cx, track.bottom() - 5.0),
                        ],
                        egui::Stroke::new(3.5, theme::servo_scroll_thumb()),
                    );
                    #[cfg(all(feature = "servo-engine", windows))]
                    if console_h > 0.0 {
                        let console_rect = egui::Rect::from_min_max(
                            egui::pos2(full.min.x, inner.max.y),
                            egui::pos2(full.max.x - SERVO_SCROLL_GUTTER, full.max.y),
                        );
                        ui.allocate_new_ui(
                            egui::UiBuilder::new()
                                .id_salt(crate::servo_engine::embedder_ids::page_console_strip())
                                .max_rect(console_rect)
                                .layout(egui::Layout::top_down(egui::Align::Min)),
                            |ui| {
                            ui.set_min_size(console_rect.size());
                            egui::Frame::default()
                                .fill(theme::content_bg())
                                .stroke(egui::Stroke::new(1.0, theme::separator()))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            egui::RichText::new(i18n::servo_page_console_header(loc))
                                                .strong(),
                                        );
                                        if ui
                                            .small_button(i18n::servo_page_console_clear(loc))
                                            .clicked()
                                        {
                                            tab.servo_console.clear();
                                        }
                                    });
                                    egui::ScrollArea::vertical()
                                        .max_height(console_h - 28.0)
                                        .stick_to_bottom(true)
                                        .show(ui, |ui| {
                                            ui.style_mut().override_font_id = Some(
                                                egui::FontId::monospace(11.0),
                                            );
                                            for (lvl, text) in &tab.servo_console {
                                                let c = servo_console_line_color(*lvl);
                                                let line = format!("[{}] {}", lvl.as_label(), text);
                                                ui.label(
                                                    egui::RichText::new(line)
                                                        .small()
                                                        .color(c),
                                                );
                                            }
                                        });
                                });
                            },
                        );
                    }
                } else {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if !tab.loading && tab.error_message.is_none() {
                                let author_hints =
                                    compute_dom_paint_hints(&tab.dom, &tab.loaded_stylesheet_parsed);
                                render_nodes(
                                    ui,
                                    loc,
                                    &tab.dom,
                                    Some(&author_hints),
                                    &mut tab.pending_link_navigation,
                                );
                            } else if !tab.loading && tab.error_message.is_some() {
                                ui.label(
                                    egui::RichText::new(i18n::suggestion_fix_url(loc))
                                        .italics()
                                        .color(theme::loading_muted()),
                                );
                            }
                        });
                }
            }
        });

        {
            let confirm = &mut self.confirm_clear_history;
            let log = &mut self.browser_log;
            #[cfg(all(feature = "servo-engine", windows))]
            let servo_viewport = &mut self.servo_viewport;
            let mut on_cleared = |cleared: internal_pages::ClearTarget| {
                if cleared != internal_pages::ClearTarget::History {
                    return;
                }
                #[cfg(all(feature = "servo-engine", windows))]
                {
                    crate::servo_engine::permission_store::remove_file();
                    servo_viewport.clear_servo_permission_memory();
                }
            };
            internal_pages::show_clear_confirm_modal(
                ctx,
                confirm,
                loc,
                i18n::internal_confirm_clear_history(loc),
                log,
                internal_pages::ClearTarget::History,
                &mut on_cleared,
            );
        }
        {
            #[cfg(all(feature = "servo-engine", windows))]
            let servo_viewport = &self.servo_viewport;
            #[cfg(all(feature = "servo-engine", windows))]
            let tabs = &mut self.tabs;
            let mut on_cleared = |cleared: internal_pages::ClearTarget| {
                if cleared != internal_pages::ClearTarget::Downloads {
                    return;
                }
                #[cfg(all(feature = "servo-engine", windows))]
                {
                    servo_viewport.clear_servo_ephemeral_queues();
                    for tab in tabs.iter_mut() {
                        tab.servo_console.clear();
                    }
                }
            };
            internal_pages::show_clear_confirm_modal(
                ctx,
                &mut self.confirm_clear_downloads,
                loc,
                i18n::internal_confirm_clear_downloads(loc),
                &mut self.browser_log,
                internal_pages::ClearTarget::Downloads,
                &mut on_cleared,
            );
        }
        internal_pages::show_reset_settings_modal(
            ctx,
            &mut self.confirm_reset_settings,
            loc,
            &mut self.settings,
        );
        let mut clear_servo_from_settings_modal = false;
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
            {
                let c = check_now.clone();
                move || c.set(true)
            },
            &mut clear_servo_from_settings_modal,
        );
        #[cfg(all(feature = "servo-engine", windows))]
        if clear_servo_from_settings_modal {
            crate::servo_engine::permission_store::remove_file();
            self.servo_viewport.clear_servo_permission_memory();
        }
        if check_now.get() {
            self.update_banner_dismissed = false;
            self.spawn_update_check();
        }

        ctx.input(|i| {
            if i.modifiers.command && i.key_pressed(egui::Key::Comma) {
                self.settings_open = true;
            }
        });

        if ctx.input(|i| i.viewport().close_requested()) {
            self.persist_last_session();
        }

        let tab_url = self.active_tab().url_input.clone();
        #[cfg(all(feature = "servo-engine", windows))]
        self.servo_viewport.sync_tonet_scheme_snapshot(
            self.loc(),
            &self.settings,
            &self.browser_log,
        );
        self.servo_viewport.tick(
            ctx,
            frame,
            self.settings.system.experimental_servo_viewport,
            tab_url.as_str(),
            self.servo_content_rect,
        );

        #[cfg(all(feature = "servo-engine", windows))]
        {
            let setting = self.settings.system.experimental_servo_viewport;
            let idx = self.active_tab;
            self.servo_viewport.sync_active_tab_from_servo(
                ctx,
                setting,
                &mut self.tabs[idx],
                &mut self.browser_log,
            );
            for action in self.servo_viewport.take_tonet_scheme_actions() {
                match action {
                    crate::servo_engine::TonetSchemeAction::ClearHistory => {
                        self.browser_log.clear_visits();
                    }
                    crate::servo_engine::TonetSchemeAction::ClearDownloads => {
                        self.browser_log.clear_downloads();
                        self.servo_viewport.clear_servo_ephemeral_queues();
                        for tab in &mut self.tabs {
                            tab.servo_console.clear();
                        }
                    }
                    crate::servo_engine::TonetSchemeAction::ClearServoSitePermissions => {
                        crate::servo_engine::permission_store::remove_file();
                        self.servo_viewport.clear_servo_permission_memory();
                    }
                }
            }
            self.servo_viewport.show_embedder_modals(
                ctx,
                setting,
                self.tabs[idx].url_input.trim(),
                self.loc(),
            );
            if let Some(url) = self.servo_viewport.take_pending_open_link_new_tonet_tab() {
                self.open_new_tab_with_url(ctx, url);
            }
            self.sync_window_title(ctx);
            self.servo_prev_content_rect = self.servo_content_rect;
        }

        window_resize::update_resize_hover_cursor(ctx, self.integrated_title_chrome);
    }
}
