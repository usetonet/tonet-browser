//! Split a tokenized stylesheet into **top-level qualified rules** (`selector { block }`).
//!
//! `@`-rules with a `{…}` block are skipped as a whole; semicolon-only `@charset` / `@import` style
//! statements are skipped up to `;`. This is a small step toward a real cascade parser.

use super::syntax::CssToken;

/// One `prelude { … }` rule at the stylesheet’s top nesting level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleQualifiedRule {
    /// Lossy text for the prelude (selectors / prelude slice before `{`).
    pub prelude_display: String,
    /// Tokens inside the `{ … }` block (excluding the outer braces).
    pub block: Vec<CssToken>,
}

/// Parse `tokens` into top-level qualified rules; skips `@` rules and semicolon-terminated at-rules.
pub fn parse_top_level_qualified_rules(tokens: &[CssToken]) -> Vec<SimpleQualifiedRule> {
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < tokens.len() {
        while i < tokens.len() && matches!(tokens[i], CssToken::Whitespace) {
            i += 1;
        }
        if i >= tokens.len() {
            break;
        }
        if matches!(tokens[i], CssToken::AtKeyword(_)) {
            i = skip_at_statement(tokens, i);
            continue;
        }
        let start = i;
        let lb = match find_first_top_level_lcurly(tokens, start) {
            Some(x) => x,
            None => break,
        };
        let prelude = &tokens[start..lb];
        if prelude.is_empty() {
            i = consume_balanced_block(tokens, lb);
            continue;
        }
        let end_block = match find_matching_rcurly(tokens, lb) {
            Some(x) => x,
            None => break,
        };
        let block = tokens[lb + 1..end_block].to_vec();
        out.push(SimpleQualifiedRule {
            prelude_display: prelude_to_display(prelude),
            block,
        });
        i = end_block + 1;
    }
    out
}

fn find_first_top_level_lcurly(tokens: &[CssToken], start: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut j = start;
    while j < tokens.len() {
        match &tokens[j] {
            CssToken::LCurly if depth == 0 => return Some(j),
            CssToken::LCurly => depth += 1,
            CssToken::RCurly => depth -= 1,
            _ => {}
        }
        j += 1;
    }
    None
}

fn find_matching_rcurly(tokens: &[CssToken], lcurly: usize) -> Option<usize> {
    if !matches!(tokens.get(lcurly), Some(CssToken::LCurly)) {
        return None;
    }
    let mut depth = 1i32;
    let mut j = lcurly + 1;
    while j < tokens.len() {
        match &tokens[j] {
            CssToken::LCurly => depth += 1,
            CssToken::RCurly => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
        j += 1;
    }
    None
}

fn consume_balanced_block(tokens: &[CssToken], lcurly: usize) -> usize {
    find_matching_rcurly(tokens, lcurly).map_or(tokens.len(), |r| r + 1)
}

fn skip_at_statement(tokens: &[CssToken], at: usize) -> usize {
    debug_assert!(matches!(tokens.get(at), Some(CssToken::AtKeyword(_))));
    let mut j = at + 1;
    while j < tokens.len() && matches!(tokens[j], CssToken::Whitespace) {
        j += 1;
    }
    // Prefer a braced block (`@media … {`) over semicolon scan so we do not stop at `;` inside the block.
    let mut k = j;
    while k < tokens.len() {
        match &tokens[k] {
            CssToken::LCurly => return consume_balanced_block(tokens, k),
            CssToken::Semicolon => return k + 1,
            _ => k += 1,
        }
    }
    tokens.len()
}

fn prelude_to_display(tokens: &[CssToken]) -> String {
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
            CssToken::LCurly | CssToken::RCurly => {}
            CssToken::Whitespace => {}
        }
    }
    s.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::css::syntax::tokenize_css;

    #[test]
    fn one_rule_body_color() {
        let t = tokenize_css("body { color: red; }");
        let rules = parse_top_level_qualified_rules(&t);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].prelude_display, "body");
        assert!(rules[0].block.iter().any(|x| matches!(x, CssToken::Ident(i) if i == "color")));
    }

    #[test]
    fn two_rules() {
        let t = tokenize_css("a { x:1; } b { y:2; }");
        let rules = parse_top_level_qualified_rules(&t);
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].prelude_display, "a");
        assert_eq!(rules[1].prelude_display, "b");
    }

    #[test]
    fn skip_charset_and_media() {
        let t = tokenize_css(r#"@charset "utf-8"; @media screen { #x { z:1 } } div {}"#);
        let rules = parse_top_level_qualified_rules(&t);
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].prelude_display, "div");
    }
}
