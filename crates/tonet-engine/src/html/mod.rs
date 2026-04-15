//! HTML parsing — incremental path toward HTML5 tree construction.
//!
//! Today: [`minimal`] implements a small streaming subset for Tonet’s UI (titles, blocks, links,
//! favicons). [`tokenizer`] is the first step toward a spec-aligned token stream; a tree builder
//! will sit on top. See `TONET_VISION.md` §5.

pub mod attributes;
pub mod minimal;
pub mod tokenizer;

pub use attributes::{parse_attributes, Attr};
