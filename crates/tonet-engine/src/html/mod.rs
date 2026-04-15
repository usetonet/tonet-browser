//! HTML parsing — incremental path toward HTML5 tree construction.
//!
//! Today: [`minimal`] implements a small streaming subset for Tonet’s UI (titles, blocks, links,
//! favicons). A tokenizer + tree builder aligned with the HTML living standard will grow here;
//! see `TONET_VISION.md` §5.

pub mod minimal;
