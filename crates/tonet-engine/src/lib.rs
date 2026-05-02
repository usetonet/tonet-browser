//! Tonet **engine** crate: thin facade over sources that live under `crates/tonet/src/` so
//! integration tests and tooling can depend on `tonet_engine` without duplicating HTML/CSS or URL policy.
//!
//! Canonical modules live next to the desktop shell (`crates/tonet`); this crate stays `#[forbid(unsafe_code)]`.
//! Roadmap: `TONET_VISION.md` (repository root).

#![forbid(unsafe_code)]

#[path = "../../tonet/src/css/mod.rs"]
pub mod css;
#[path = "../../tonet/src/document_url.rs"]
pub mod document_url;
#[path = "../../tonet/src/html/mod.rs"]
pub mod html;
pub mod js;
#[path = "../../tonet/src/limits.rs"]
pub mod limits;
pub mod navigation;
#[path = "../../tonet/src/policy.rs"]
pub mod policy;

pub use limits::EngineLimits;
