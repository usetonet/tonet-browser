//! Parse **`property: value`** declarations inside a `{ … }` block (no nested `{}`).

use super::simple_rules::SimpleQualifiedRule;
use super::syntax::CssToken;

/// One declaration inside a style rule block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleDeclaration {
    pub property: String,
    /// Lossy text for the value (from the colon up to `;` or EOF).
    pub value_display: String,
}

/// Split `tokens` (contents of a `{ … }` block) into declarations separated by `;`.
///
/// Nested blocks are not supported; stray `;` before a `:` skips to the next segment.
pub fn parse_declaration_block(tokens: &[CssToken]) -> Vec<SimpleDeclaration> {
    let mut out = Vec::new();
    let mut i = 0usize;
    'scan: while i < tokens.len() {
        while i < tokens.len() && matches!(tokens[i], CssToken::Whitespace) {
            i += 1;
        }
        if i >= tokens.len() {
            break;
        }
        let name_start = i;
        let mut colon = None;
        let mut j = i;
        while j < tokens.len() {
            match &tokens[j] {
                CssToken::Colon => {
                    colon = Some(j);
                    break;
                }
                CssToken::Semicolon => {
                    i = j + 1;
                    continue 'scan;
                }
                _ => j += 1,
            }
        }
        let Some(c) = colon else {
            break;
        };
        let property = slice_display(&tokens[name_start..c]).trim().to_string();
        if property.is_empty() {
            i = c + 1;
            continue;
        }
        j = c + 1;
        while j < tokens.len() && matches!(tokens[j], CssToken::Whitespace) {
            j += 1;
        }
        let val_start = j;
        while j < tokens.len() && !matches!(tokens[j], CssToken::Semicolon) {
            j += 1;
        }
        let value_display = slice_display(&tokens[val_start..j]).trim().to_string();
        out.push(SimpleDeclaration {
            property,
            value_display,
        });
        i = if j < tokens.len() { j + 1 } else { j };
    }
    out
}

/// Declarations for one [`SimpleQualifiedRule`].
pub fn declarations_for_rule(rule: &SimpleQualifiedRule) -> Vec<SimpleDeclaration> {
    parse_declaration_block(&rule.block)
}

/// One qualified rule with its parsed declaration list (no selector engine yet).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedQualifiedRule {
    pub prelude_display: String,
    pub declarations: Vec<SimpleDeclaration>,
}

/// Map each [`SimpleQualifiedRule`] to prelude + declarations.
pub fn parse_qualified_rules_declarations(rules: &[SimpleQualifiedRule]) -> Vec<ParsedQualifiedRule> {
    rules
        .iter()
        .map(|r| ParsedQualifiedRule {
            prelude_display: r.prelude_display.clone(),
            declarations: declarations_for_rule(r),
        })
        .collect()
}

/// Per-stylesheet URL: rules with parsed declaration blocks.
pub fn parse_stylesheet_bundle_rule_declarations(
    bundle: &[(String, Vec<SimpleQualifiedRule>)],
) -> Vec<(String, Vec<ParsedQualifiedRule>)> {
    bundle
        .iter()
        .map(|(url, rules)| {
            (
                url.clone(),
                parse_qualified_rules_declarations(rules),
            )
        })
        .collect()
}

fn slice_display(tokens: &[CssToken]) -> String {
    let mut s = String::new();
    let mut need_space = false;
    for t in tokens {
        if matches!(t, CssToken::Whitespace) {
            need_space = true;
            continue;
        }
        if need_space && !s.is_empty() {
            s.push(' ');
        }
        need_space = false;
        match t {
            CssToken::Ident(x) => s.push_str(x),
            CssToken::Hash(x) => {
                s.push('#');
                s.push_str(x);
            }
            CssToken::String(x) => {
                s.push('"');
                s.push_str(x);
                s.push('"');
            }
            CssToken::AtKeyword(x) => {
                s.push('@');
                s.push_str(x);
            }
            CssToken::Delim(c) => s.push(*c),
            CssToken::Colon => s.push(':'),
            CssToken::Comma => s.push(','),
            CssToken::Semicolon => s.push(';'),
            CssToken::LParen => s.push('('),
            CssToken::RParen => s.push(')'),
            CssToken::LBracket => s.push('['),
            CssToken::RBracket => s.push(']'),
            CssToken::LCurly | CssToken::RCurly | CssToken::Whitespace => {}
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::syntax::tokenize_css;

    #[test]
    fn color_red_semicolon() {
        let t = tokenize_css("color: red;");
        let d = parse_declaration_block(&t);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].property, "color");
        assert_eq!(d[0].value_display, "red");
    }

    #[test]
    fn two_declarations() {
        let t = tokenize_css("color: red; margin: 0 1px;");
        let d = parse_declaration_block(&t);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].property, "color");
        assert_eq!(d[1].property, "margin");
        assert_eq!(d[1].value_display, "0 1px");
    }

    #[test]
    fn from_rule_block() {
        let t = tokenize_css("body { font-size: 14px; color: navy }");
        let rules = crate::css::parse_top_level_qualified_rules(&t);
        assert_eq!(rules.len(), 1);
        let d = declarations_for_rule(&rules[0]);
        assert_eq!(d.len(), 2);
        assert_eq!(d[0].property, "font-size");
        assert_eq!(d[1].property, "color");
    }

    #[test]
    fn bundle_rule_declarations() {
        let t = tokenize_css("p { margin: 0 } h1 { font-weight: bold }");
        let rules = crate::css::parse_top_level_qualified_rules(&t);
        let bundle = vec![("https://x/a.css".into(), rules)];
        let parsed = parse_stylesheet_bundle_rule_declarations(&bundle);
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].1.len(), 2);
        assert_eq!(parsed[0].1[0].prelude_display, "p");
        assert_eq!(parsed[0].1[0].declarations.len(), 1);
        assert_eq!(parsed[0].1[1].prelude_display, "h1");
    }
}
