//! Incremental **HTML tokenizer** toward WHATWG / HTML5 conformance.
//!
//! Emits text, comments, and tags; start-tag attributes are parsed via [`super::attributes`].

use super::attributes::parse_attributes;
use super::entities::decode_html_entities;

/// One lexical token from the input stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Character data (UTF-8; character references like `&amp;` are decoded).
    Text(String),
    /// `<tag …>`.
    StartTag {
        name: String,
        self_closing: bool,
        attrs: Vec<super::attributes::Attr>,
    },
    /// `</tag>`
    EndTag { name: String },
    /// `<!-- … -->` inner text (without delimiters).
    Comment(String),
    EndOfFile,
}

/// Tokenize `input` into a flat token stream (lossy recovery on malformed markup).
pub fn tokenize(input: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut it = input.chars().peekable();
    let mut text_buf = String::new();

    fn flush_text(out: &mut Vec<Token>, buf: &mut String) {
        if !buf.is_empty() {
            let raw = std::mem::take(buf);
            out.push(Token::Text(decode_html_entities(&raw)));
        }
    }

    while let Some(&c) = it.peek() {
        if c != '<' {
            text_buf.push(it.next().unwrap());
            continue;
        }
        flush_text(&mut out, &mut text_buf);
        it.next(); // '<'

        match it.peek().copied() {
            Some('!') => {
                it.next(); // '!'
                if it.peek() == Some(&'-') {
                    it.next();
                    if it.peek() == Some(&'-') {
                        it.next();
                        let mut comment = String::new();
                        while let Some(ch) = it.next() {
                            if ch == '-' && it.peek() == Some(&'-') {
                                it.next();
                                if it.peek() == Some(&'>') {
                                    it.next();
                                    break;
                                }
                                comment.push_str("--");
                                continue;
                            }
                            comment.push(ch);
                        }
                        out.push(Token::Comment(comment));
                    } else {
                        // `<!-` without second `-`: bogus, drop through
                        while let Some(ch) = it.next() {
                            if ch == '>' {
                                break;
                            }
                        }
                    }
                } else {
                    // Bogus declaration: consume until next `>`
                    while let Some(ch) = it.next() {
                        if ch == '>' {
                            break;
                        }
                    }
                }
            }
            Some('/') => {
                it.next();
                let name = read_tag_name(&mut it);
                skip_to_gt(&mut it);
                if !name.is_empty() {
                    out.push(Token::EndTag { name });
                }
            }
            Some(ch) if ch.is_ascii_alphabetic() || ch == '?' => {
                let name = read_tag_name(&mut it);
                let (self_closing, attrs_raw) = read_until_tag_close(&mut it);
                if !name.is_empty() {
                    let attrs = parse_attributes(&attrs_raw);
                    out.push(Token::StartTag {
                        name,
                        self_closing,
                        attrs,
                    });
                }
            }
            _ => {
                text_buf.push('<');
            }
        }
    }

    flush_text(&mut out, &mut text_buf);
    out.push(Token::EndOfFile);
    out
}

fn read_tag_name(it: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut s = String::new();
    while let Some(&c) = it.peek() {
        if c.is_ascii_alphanumeric() || c == '-' || c == ':' {
            s.push(it.next().unwrap().to_ascii_lowercase());
        } else {
            break;
        }
    }
    s
}

/// After the tag name: read until `>`; return (self_closing, attribute source between name and `>`).
fn read_until_tag_close(it: &mut std::iter::Peekable<std::str::Chars<'_>>) -> (bool, String) {
    let mut raw = String::new();
    while let Some(c) = it.next() {
        if c == '>' {
            let t = raw.trim();
            if t.ends_with('/') {
                let attrs = t[..t.len().saturating_sub(1)].trim();
                return (true, attrs.to_string());
            }
            return (false, t.to_string());
        }
        raw.push(c);
    }
    (false, raw.trim().to_string())
}

fn skip_to_gt(it: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while let Some(c) = it.next() {
        if c == '>' {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_yields_eof() {
        let t = tokenize("");
        assert_eq!(t, vec![Token::EndOfFile]);
    }

    #[test]
    fn plain_text() {
        let t = tokenize("hello");
        assert_eq!(
            t,
            vec![Token::Text("hello".into()), Token::EndOfFile]
        );
    }

    #[test]
    fn start_and_end_p() {
        let t = tokenize("<p>x</p>");
        assert_eq!(
            t,
            vec![
                Token::StartTag {
                    name: "p".into(),
                    self_closing: false,
                    attrs: vec![],
                },
                Token::Text("x".into()),
                Token::EndTag { name: "p".into() },
                Token::EndOfFile,
            ]
        );
    }

    #[test]
    fn comment_only() {
        let t = tokenize("<!-- hi -->");
        assert_eq!(
            t,
            vec![Token::Comment(" hi ".into()), Token::EndOfFile]
        );
    }

    #[test]
    fn comment_between_text() {
        let t = tokenize("a<!--b-->c");
        assert_eq!(
            t,
            vec![
                Token::Text("a".into()),
                Token::Comment("b".into()),
                Token::Text("c".into()),
                Token::EndOfFile,
            ]
        );
    }

    #[test]
    fn self_closing_br() {
        let t = tokenize("<br/>");
        assert_eq!(
            t,
            vec![
                Token::StartTag {
                    name: "br".into(),
                    self_closing: true,
                    attrs: vec![],
                },
                Token::EndOfFile,
            ]
        );
    }

    #[test]
    fn self_closing_br_with_spaces() {
        let t = tokenize("<br />");
        assert_eq!(
            t,
            vec![
                Token::StartTag {
                    name: "br".into(),
                    self_closing: true,
                    attrs: vec![],
                },
                Token::EndOfFile,
            ]
        );
    }

    #[test]
    fn text_decodes_character_references() {
        let t = tokenize("<p>a&amp;b&lt;c</p>");
        assert!(
            t.iter().any(|x| matches!(x, Token::Text(s) if s == "a&b<c")),
            "{t:?}"
        );
    }

    #[test]
    fn attribute_value_decodes_entities() {
        let t = tokenize(r#"<span title="a&quot;b">x</span>"#);
        let st = t.iter().find_map(|x| match x {
            Token::StartTag { name, attrs, .. } if name == "span" => Some(attrs),
            _ => None,
        })
        .unwrap();
        assert_eq!(
            st.iter().find(|a| a.name == "title").map(|a| a.value.as_str()),
            Some("a\"b")
        );
    }

    #[test]
    fn tag_with_parsed_attrs() {
        use crate::html::attributes::Attr;
        let t = tokenize(r#"<a href="/z" class=x>y</a>"#);
        assert_eq!(
            t,
            vec![
                Token::StartTag {
                    name: "a".into(),
                    self_closing: false,
                    attrs: vec![
                        Attr {
                            name: "href".into(),
                            value: "/z".into(),
                        },
                        Attr {
                            name: "class".into(),
                            value: "x".into(),
                        },
                    ],
                },
                Token::Text("y".into()),
                Token::EndTag { name: "a".into() },
                Token::EndOfFile,
            ]
        );
    }
}
