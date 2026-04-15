//! Tonet **engine** contracts: budgets, navigation rules, and size policy.
//!
//! UI lives in `tonet`; this crate stays dependency-light and `#[forbid(unsafe_code)]`.
//! Workspace vision and gates: `TONET_VISION.md` (repository root).

#![forbid(unsafe_code)]

pub mod document_url;
pub mod limits;
pub mod navigation;
pub mod policy;

pub use limits::EngineLimits;
