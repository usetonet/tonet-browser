//! CSS syntax, selectors, cascade, and computed styles.
//!
//! **Phase 1:** not wired to layout; Tonet renders a minimal DOM without author stylesheets.
//! [`syntax`] provides a first tokenizer slice; planned layers: selectors → cascade → box model
//! (see `TONET_VISION.md` §5).

pub mod syntax;

pub use syntax::{tokenize_css, CssToken};
