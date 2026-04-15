//! One browser tab: address, history, DOM, and in-flight fetch state.

use std::sync::mpsc;

use crate::parser::{DomNode, DomNodeType};

pub struct PageFetchData {
    pub nodes: Vec<DomNode>,
    pub favicon_candidates: Vec<String>,
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
    /// True while this tab should display the New Tab page.
    /// Cleared only when an actual navigation starts (Enter pressed).
    pub show_new_tab: bool,
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
            show_new_tab: is_empty,
        }
    }

    pub fn is_new_tab(&self) -> bool {
        self.show_new_tab
    }

    /// Drop any in-flight fetch so results cannot apply to the wrong tab after switching.
    pub fn cancel_in_flight(&mut self) {
        self.fetch_rx = None;
        self.favicon_fetch_rx = None;
        self.loading = false;
        self.pending_nav = None;
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
                self.history.truncate(self.hist_index.saturating_add(1));
                self.history.push(HistoryEntry { url, nodes });
                self.hist_index = self.history.len().saturating_sub(1);
            }
        }
    }
}
