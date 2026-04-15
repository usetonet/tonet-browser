//! CSS syntax, selectors, cascade, and computed styles.
//!
//! **Phase 1:** not wired to layout; Tonet renders a minimal DOM without author stylesheets.
//! [`syntax`] provides a first tokenizer slice; [`simple_rules`] splits top-level `{…}` rules.
//! Planned layers: selectors → cascade → box model (see `TONET_VISION.md` §5).

pub mod simple_rules;
pub mod syntax;

pub use simple_rules::{
    parse_stylesheet_bundle_to_rules, parse_top_level_qualified_rules, SimpleQualifiedRule,
};
pub use syntax::{tokenize_css, tokenize_stylesheet_bundle, CssToken};
