//! One browser tab: address, history, DOM, and in-flight fetch state.

use std::sync::mpsc;

use crate::css::{CssToken, ParsedQualifiedRule, SimpleQualifiedRule};

use crate::parser::{DomNode, DomNodeType};

pub struct PageFetchData {
    pub nodes: Vec<DomNode>,
    pub favicon_candidates: Vec<String>,
    /// Absolute author stylesheet URLs from `<link rel=stylesheet>` (not fetched yet).
    pub stylesheet_candidates: Vec<String>,
    /// Raw UTF-8 HTML from the network (used for optional on-disk snapshots).
    pub raw_html: String,
}

pub type FetchResult = Result<PageFetchData, String>;

#[derive(Debug, Clone, Copy)]
pub enum NavigateIntent {
    NewPage,
    Reload,
}

#[derive(Clone)]
pub struct HistoryEntry {
    pub url: String,
    pub nodes: Vec<DomNode>,
    pub stylesheet_urls: Vec<String>,
}

/// New tabs start empty (shows the New Tab page).
pub const DEFAULT_HOME_URL: &str = "";

pub struct Tab {
    /// If set after rendering the page, navigate to this URL on the next frame.
    pub pending_link_navigation: Option<String>,
    pub url_input: String,
    pub loading: bool,
    pub error_message: Option<String>,
    pub dom: Vec<DomNode>,
    pub fetch_rx: Option<mpsc::Receiver<FetchResult>>,
    pub history: Vec<HistoryEntry>,
    pub hist_index: usize,
    pub pending_nav: Option<(String, NavigateIntent)>,
    /// Per-page favicon URI registered with egui. Empty means "no favicon yet".
    pub favicon_uri: String,
    /// Async favicon fetch result.
    pub favicon_fetch_rx: Option<mpsc::Receiver<Option<Vec<u8>>>>,
    /// Async author stylesheet bodies `(url, css_text)` after navigation.
    pub stylesheet_fetch_rx: Option<mpsc::Receiver<Vec<(String, String)>>>,
    /// Last successfully parsed author stylesheet URLs for this document (empty until a fetch completes).
    pub stylesheet_urls: Vec<String>,
    /// Fetched stylesheet bodies for [`Self::stylesheet_urls`] (subset that returned 200 + size OK).
    /// Not yet consumed by a style engine; kept for the next cascade/layout milestone.
    #[allow(dead_code)]
    pub loaded_stylesheets: Vec<(String, String)>,
    /// Tokenized author stylesheets (parallel to [`Self::loaded_stylesheets`] text).
    #[allow(dead_code)]
    pub loaded_stylesheet_tokens: Vec<(String, Vec<CssToken>)>,
    /// Top-level qualified rules per stylesheet URL (from [`crate::css::parse_stylesheet_bundle_to_rules`]).
    #[allow(dead_code)]
    pub loaded_stylesheet_rules: Vec<(String, Vec<SimpleQualifiedRule>)>,
    /// Same rules as [`Self::loaded_stylesheet_rules`] with `property: value` lists per block.
    #[allow(dead_code)]
    pub loaded_stylesheet_parsed: Vec<(String, Vec<ParsedQualifiedRule>)>,
    /// True while this tab should display the New Tab page.
    /// Cleared only when an actual navigation starts (Enter pressed).
    pub show_new_tab: bool,

    /// Document title reported by the Servo embedder (Windows experimental viewport).
    #[cfg(all(feature = "servo-engine", windows))]
    pub servo_document_title: Option<String>,
    /// `(can_go_back, can_go_forward)` from Servo when this tab uses the native Servo viewport.
    #[cfg(all(feature = "servo-engine", windows))]
    pub servo_chrome_nav: Option<(bool, bool)>,
    /// Recent `console.*` output from the Servo `WebView` (active tab only; drained from the host each frame).
    #[cfg(all(feature = "servo-engine", windows))]
    pub servo_console: Vec<(ServoConsoleLevel, String)>,
}

/// [`servo::ConsoleLogLevel`] mirrored here so `Tab` does not depend on the optional `servo` crate.
#[cfg(all(feature = "servo-engine", windows))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ServoConsoleLevel {
    Log,
    Debug,
    Info,
    Warn,
    Error,
    Trace,
}

#[cfg(all(feature = "servo-engine", windows))]
impl ServoConsoleLevel {
    pub fn as_label(self) -> &'static str {
        match self {
            Self::Log => "LOG",
            Self::Debug => "DEBUG",
            Self::Info => "INFO",
            Self::Warn => "WARN",
            Self::Error => "ERROR",
            Self::Trace => "TRACE",
        }
    }
}

impl Tab {
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
        let is_empty = url.is_empty();
        Self {
            pending_link_navigation: None,
            url_input: url,
            loading: false,
            error_message: None,
            dom: Vec::new(),
            fetch_rx: None,
            history: Vec::new(),
            hist_index: 0,
            pending_nav: None,
            favicon_uri: String::new(),
            favicon_fetch_rx: None,
            stylesheet_fetch_rx: None,
            stylesheet_urls: Vec::new(),
            loaded_stylesheets: Vec::new(),
            loaded_stylesheet_tokens: Vec::new(),
            loaded_stylesheet_rules: Vec::new(),
            loaded_stylesheet_parsed: Vec::new(),
            show_new_tab: is_empty,
            #[cfg(all(feature = "servo-engine", windows))]
            servo_document_title: None,
            #[cfg(all(feature = "servo-engine", windows))]
            servo_chrome_nav: None,
            #[cfg(all(feature = "servo-engine", windows))]
            servo_console: Vec::new(),
        }
    }

    pub fn is_new_tab(&self) -> bool {
        self.show_new_tab
    }

    /// Drop any in-flight fetch so results cannot apply to the wrong tab after switching.
    pub fn cancel_in_flight(&mut self) {
        self.fetch_rx = None;
        self.favicon_fetch_rx = None;
        self.stylesheet_fetch_rx = None;
        self.loaded_stylesheets.clear();
        self.loaded_stylesheet_tokens.clear();
        self.loaded_stylesheet_rules.clear();
        self.loaded_stylesheet_parsed.clear();
        self.loading = false;
        self.pending_nav = None;
    }

    /// Append a Servo console line; keeps a bounded tail for the in-page console strip.
    #[cfg(all(feature = "servo-engine", windows))]
    pub fn push_servo_console_line(&mut self, level: ServoConsoleLevel, message: String) {
        const MAX_LINES: usize = 400;
        self.servo_console.push((level, message));
        while self.servo_console.len() > MAX_LINES {
            self.servo_console.remove(0);
        }
    }

    pub fn doc_title_trimmed(&self) -> Option<&str> {
        self.dom
            .iter()
            .find(|n| n.kind == DomNodeType::Title)
            .map(|n| n.text.as_str())
            .filter(|s| !s.trim().is_empty())
    }

    pub fn apply_successful_navigation(&mut self, nodes: Vec<DomNode>) {
        let Some((url, intent)) = self.pending_nav.take() else {
            self.dom = nodes;
            return;
        };

        let stylesheet_urls = self.stylesheet_urls.clone();
        self.dom = nodes.clone();

        match intent {
            NavigateIntent::Reload => {
                if self.history.is_empty() {
                    self.history.push(HistoryEntry {
                        url,
                        nodes,
                        stylesheet_urls,
                    });
                    self.hist_index = 0;
                } else if self.hist_index < self.history.len() {
                    self.history[self.hist_index] = HistoryEntry {
                        url,
                        nodes,
                        stylesheet_urls,
                    };
                } else {
                    self.history.push(HistoryEntry {
                        url,
                        nodes,
                        stylesheet_urls,
                    });
                    self.hist_index = self.history.len() - 1;
                }
            }
            NavigateIntent::NewPage => {
                self.history.truncate(self.hist_index.saturating_add(1));
                self.history.push(HistoryEntry {
                    url,
                    nodes,
                    stylesheet_urls,
                });
                self.hist_index = self.history.len().saturating_sub(1);
            }
        }
    }
}
