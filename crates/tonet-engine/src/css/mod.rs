//! CSS syntax, selectors, cascade, and computed styles.
//!
//! **Phase 1:** not wired to layout; Tonet renders a minimal DOM without author stylesheets.
//! [`syntax`] provides a first tokenizer slice; [`simple_rules`] splits top-level `{…}` rules;
//! [`declarations`] parses `property: value` inside each block.
//! Planned layers: selectors → cascade → box model (see `TONET_VISION.md` §5).

pub mod declarations;
pub mod simple_rules;
pub mod syntax;

pub use declarations::{
    declarations_for_rule, parse_declaration_block, parse_qualified_rules_declarations,
    parse_stylesheet_bundle_rule_declarations, ParsedQualifiedRule, SimpleDeclaration,
};
pub use simple_rules::{
    parse_stylesheet_bundle_to_rules, parse_top_level_qualified_rules, SimpleQualifiedRule,
};
pub use syntax::{tokenize_css, tokenize_stylesheet_bundle, CssToken};
