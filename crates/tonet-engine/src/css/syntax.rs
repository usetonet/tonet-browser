//! Minimal CSS **syntax** tokenizer toward author stylesheets (`TONET_VISION.md` §5).
//!
//! Not wired to layout or the HTML pipeline yet; incremental tests lock behavior.

/// Token from a simplified stylesheet scan (comments, identifiers, punctuation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CssToken {
    Whitespace,
    Ident(String),
    Hash(String),
    AtKeyword(String),
    String(String),
    LCurly,
    RCurly,
    LParen,
    RParen,
    LBracket,
    RBracket,
    Colon,
    Semicolon,
    Comma,
    Delim(char),
}

/// Tokenize `input` for basic rulesets (ASCII‑centric; enough for smoke tests and growth).
pub fn tokenize_css(input: &str) -> Vec<CssToken> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut out = Vec::new();

    while i < chars.len() {
        let c = chars[i];

        if c.is_whitespace() {
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                i += 1;
            }
            out.push(CssToken::Whitespace);
            continue;
        }

        if c == '/' && chars.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < chars.len() && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < chars.len() {
                i += 2;
            } else {
                i = chars.len();
            }
            continue;
        }

        if c == '"' || c == '\'' {
            let quote = c;
            i += 1;
            let mut s = String::new();
            while i < chars.len() {
                let ch = chars[i];
                if ch == '\\' && i + 1 < chars.len() {
                    s.push(chars[i + 1]);
                    i += 2;
                    continue;
                }
                if ch == quote {
                    i += 1;
                    break;
                }
                s.push(ch);
                i += 1;
            }
            out.push(CssToken::String(s));
            continue;
        }

        if c == '#' {
            i += 1;
            let (name, ni) = consume_name(&chars, i);
            i = ni;
            if name.is_empty() {
                out.push(CssToken::Delim('#'));
            } else {
                out.push(CssToken::Hash(name));
            }
            continue;
        }

        if c == '@' {
            i += 1;
            let (name, ni) = consume_name(&chars, i);
            i = ni;
            if name.is_empty() {
                out.push(CssToken::Delim('@'));
            } else {
                out.push(CssToken::AtKeyword(name));
            }
            continue;
        }

        if could_start_name(&chars, i) {
            let (name, ni) = consume_name(&chars, i);
            i = ni;
            out.push(CssToken::Ident(name));
            continue;
        }

        i += 1;
        match c {
            '{' => out.push(CssToken::LCurly),
            '}' => out.push(CssToken::RCurly),
            '(' => out.push(CssToken::LParen),
            ')' => out.push(CssToken::RParen),
            '[' => out.push(CssToken::LBracket),
            ']' => out.push(CssToken::RBracket),
            ':' => out.push(CssToken::Colon),
            ';' => out.push(CssToken::Semicolon),
            ',' => out.push(CssToken::Comma),
            other => out.push(CssToken::Delim(other)),
        }
    }

    out
}

/// Whether `chars[i]` can start a CSS identifier (`-a`, `--foo`, `body`, …).
fn could_start_name(chars: &[char], i: usize) -> bool {
    match chars.get(i).copied() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => true,
        Some('-') => match chars.get(i + 1).copied() {
            Some('-') => chars
                .get(i + 2)
                .map_or(false, |c| c.is_ascii_alphabetic() || *c == '_'),
            Some(c2) if c2.is_ascii_alphabetic() || c2 == '_' => true,
            _ => false,
        },
        _ => false,
    }
}

#[inline]
fn is_name_char(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == '-' || c.is_ascii_digit()
}

/// Consume a CSS name (subset: ASCII letters, digits, `_`, `-`, and leading `--` for custom props).
fn consume_name(chars: &[char], mut i: usize) -> (String, usize) {
    let mut s = String::new();
    while i < chars.len() {
        let c = chars[i];
        if c == '-' {
            s.push(c);
            i += 1;
            continue;
        }
        if is_name_char(c) {
            s.push(c);
            i += 1;
            continue;
        }
        break;
    }
    (s, i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comment_only_yields_empty() {
        assert!(tokenize_css("/* nothing */").is_empty());
    }

    #[test]
    fn simple_rule() {
        let t = tokenize_css("body { color: red; }");
        assert!(
            matches!(t.first(), Some(CssToken::Ident(s)) if s == "body"),
            "{t:?}"
        );
        assert!(t.contains(&CssToken::LCurly));
        assert!(t.contains(&CssToken::RCurly));
        assert!(t.contains(&CssToken::Colon));
        assert!(t.contains(&CssToken::Semicolon));
        let idents: Vec<_> = t
            .iter()
            .filter_map(|x| match x {
                CssToken::Ident(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();
        assert!(idents.contains(&"body"));
        assert!(idents.contains(&"color"));
        assert!(idents.contains(&"red"));
    }

    #[test]
    fn hash_and_at_keyword() {
        let t = tokenize_css("@media screen { #id { } }");
        assert!(matches!(t[0], CssToken::AtKeyword(ref s) if s == "media"));
        assert!(t
            .iter()
            .any(|x| matches!(x, CssToken::Hash(s) if s == "id")));
    }

    #[test]
    fn string_with_escape() {
        let t = tokenize_css(r#"a { content: "x\"y"; }"#);
        let s = t.iter().find_map(|x| match x {
            CssToken::String(s) => Some(s.clone()),
            _ => None,
        });
        assert_eq!(s.as_deref(), Some("x\"y")); // `x` + `"` + `y`
    }

    #[test]
    fn custom_property_ident() {
        let t = tokenize_css(".box { --bg: #fff; }");
        assert!(t
            .iter()
            .any(|x| matches!(x, CssToken::Ident(s) if s == "--bg")));
    }
}
