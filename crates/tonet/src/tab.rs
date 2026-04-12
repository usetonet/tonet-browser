//! One browser tab: address, history, DOM, and in-flight fetch state.

use std::sync::mpsc;

use crate::parser::{DomNode, DomNodeType};

pub type FetchResult = Result<Vec<DomNode>, String>;

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

/// Default URL for a new tab (same as first-run home).
pub const DEFAULT_HOME_URL: &str = "https://usetonet.com";

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
}

impl Tab {
    pub fn new(url: impl Into<String>) -> Self {
        let url = url.into();
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
        }
    }

    /// Drop any in-flight fetch so results cannot apply to the wrong tab after switching.
    pub fn cancel_in_flight(&mut self) {
        self.fetch_rx = None;
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
