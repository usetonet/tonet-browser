//! HTML parsing — incremental path toward HTML5 tree construction.
//!
//! Today: [`minimal`] implements a small streaming subset for Tonet’s UI (titles, blocks, links,
//! favicons). [`tokenizer`] + [`tree_builder`] build a DOM-shaped tree from markup; full HTML5
//! algorithms are still TBD. See `TONET_VISION.md` §5.

pub mod attributes;
pub mod entities;
pub mod minimal;
pub mod tokenizer;
pub mod tree_builder;

pub use attributes::{parse_attributes, Attr};
pub use entities::decode_html_entities;
pub use tree_builder::{DocumentFragment, ElementNode, Node};
