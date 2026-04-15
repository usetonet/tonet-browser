//! Tonet **engine**: networking policy, HTML/CSS/JS surface (incremental), and document budgets.
//!
//! UI lives in `tonet`; this crate stays `#[forbid(unsafe_code)]`.
//! Roadmap: `TONET_VISION.md` (repository root).

#![forbid(unsafe_code)]

pub mod css;
pub mod document_url;
pub mod html;
pub mod js;
pub mod limits;
pub mod navigation;
pub mod policy;

pub use limits::EngineLimits;
