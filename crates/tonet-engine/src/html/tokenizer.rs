//! Incremental **HTML tokenizer** toward WHATWG / HTML5 conformance.
//!
//! This is not a full living-standard implementation yet: it tokenizes a useful subset
//! (text, start/end tags, comments) and skips attribute lists into a lump for now — enough to
//! drive tests and a future tree builder.

/// One lexical token from the input stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    /// Character data (decoded as UTF-8; `&amp;` etc. are **not** resolved yet).
    Text(String),
    /// `<tag …>`; `attrs_raw` is the slice between the tag name and `>` (trimmed), for later parsing.
    StartTag {
        name: String,
        self_closing: bool,
        /// Unparsed attribute source (empty until a dedicated attribute tokenizer exists).
        attrs_raw: String,
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
            out.push(Token::Text(std::mem::take(buf)));
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
                    out.push(Token::StartTag {
                        name,
                        self_closing,
                        attrs_raw,
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
                    attrs_raw: String::new(),
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
                    attrs_raw: String::new(),
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
                    attrs_raw: String::new(),
                },
                Token::EndOfFile,
            ]
        );
    }

    #[test]
    fn tag_with_unparsed_attrs() {
        let t = tokenize(r#"<a href="/z" class=x>y</a>"#);
        assert_eq!(
            t,
            vec![
                Token::StartTag {
                    name: "a".into(),
                    self_closing: false,
                    attrs_raw: r#"href="/z" class=x"#.into(),
                },
                Token::Text("y".into()),
                Token::EndTag { name: "a".into() },
                Token::EndOfFile,
            ]
        );
    }
}
