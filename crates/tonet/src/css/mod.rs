//! CSS syntax, selectors, cascade, and computed styles.
//!
//! **Phase 1:** layout is still minimal; the shell can apply a **bounded** author subset (see
//! [`author_cascade`]) while the full box model remains future work.
//! [`syntax`] provides a first tokenizer slice; [`simple_rules`] splits top-level `{…}` rules;
//! [`declarations`] parses `property: value` inside each block; [`author_cascade`] applies a tiny
//! selector model (type / class / id, `html`/`body` document defaults, specificity + source order)
//! for shell integration.
//! Planned layers: full selectors → cascade → box model (see `TONET_VISION.md` §5).

pub mod author_cascade;
pub mod declarations;
pub mod simple_rules;
pub mod syntax;

pub use author_cascade::{
    cascade_document_defaults, cascade_element_rules, cascade_simple_type_rules,
    parse_simple_prelude, prelude_matches_simple_type, SimpleSelectorPrelude,
};
pub use declarations::{
    declarations_for_rule, parse_declaration_block, parse_qualified_rules_declarations,
    parse_stylesheet_bundle_rule_declarations, ParsedQualifiedRule, SimpleDeclaration,
};
pub use simple_rules::{
    parse_stylesheet_bundle_to_rules, parse_top_level_qualified_rules, SimpleQualifiedRule,
};
pub use syntax::{tokenize_css, tokenize_stylesheet_bundle, CssToken};
