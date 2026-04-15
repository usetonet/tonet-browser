//! HTML **attribute** parsing from the raw string between a start-tag name and `>`.
//!
//! Covers common cases (quoted / unquoted / boolean). Character references in values are decoded
//! via [`super::entities::decode_html_entities`].

use super::entities::decode_html_entities;

/// One `name`=`value` pair from a start tag (`value` empty for boolean attributes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attr {
    pub name: String,
    pub value: String,
}

/// Parse `attributes` into a list. Names are ASCII-lowercased.
pub fn parse_attributes(raw: &str) -> Vec<Attr> {
    let mut it = raw.trim().chars().peekable();
    let mut out = Vec::new();

    loop {
        skip_ws(&mut it);
        if it.peek().is_none() {
            break;
        }

        let name = read_attr_name(&mut it);
        if name.is_empty() {
            // Malformed: drop one character and try to resync (e.g. stray `=`)
            if it.next().is_none() {
                break;
            }
            continue;
        }

        skip_ws(&mut it);
        if it.peek() == Some(&'=') {
            it.next();
            skip_ws(&mut it);
            let value = decode_html_entities(&read_attr_value(&mut it));
            out.push(Attr { name, value });
        } else {
            out.push(Attr {
                name,
                value: String::new(),
            });
        }
    }

    out
}

fn skip_ws(it: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    while it.peek().map_or(false, |c| c.is_whitespace()) {
        it.next();
    }
}

fn read_attr_name(it: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut s = String::new();
    while let Some(&c) = it.peek() {
        if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ':' {
            s.push(it.next().unwrap().to_ascii_lowercase());
        } else {
            break;
        }
    }
    s
}

fn read_attr_value(it: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    match it.peek() {
        Some('"') => {
            it.next();
            let mut s = String::new();
            while let Some(c) = it.next() {
                if c == '"' {
                    break;
                }
                s.push(c);
            }
            s
        }
        Some('\'') => {
            it.next();
            let mut s = String::new();
            while let Some(c) = it.next() {
                if c == '\'' {
                    break;
                }
                s.push(c);
            }
            s
        }
        _ => {
            let mut s = String::new();
            while let Some(&c) = it.peek() {
                if c.is_whitespace() {
                    break;
                }
                s.push(it.next().unwrap());
            }
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty() {
        assert!(parse_attributes("").is_empty());
    }

    #[test]
    fn href_double_quoted() {
        let a = parse_attributes(r#"href="/z""#);
        assert_eq!(
            a,
            vec![Attr {
                name: "href".into(),
                value: "/z".into(),
            }]
        );
    }

    #[test]
    fn two_unquoted_and_quoted() {
        let a = parse_attributes(r#"href=/z class=x data-y="1""#);
        assert_eq!(a.len(), 3);
        assert_eq!(a[0].name, "href");
        assert_eq!(a[0].value, "/z");
        assert_eq!(a[1].name, "class");
        assert_eq!(a[1].value, "x");
        assert_eq!(a[2].name, "data-y");
        assert_eq!(a[2].value, "1");
    }

    #[test]
    fn boolean_hidden() {
        let a = parse_attributes("hidden");
        assert_eq!(
            a,
            vec![Attr {
                name: "hidden".into(),
                value: String::new(),
            }]
        );
    }

    #[test]
    fn value_with_amp_entity() {
        let a = parse_attributes(r#"title="Tom &amp; Jerry""#);
        assert_eq!(
            a,
            vec![Attr {
                name: "title".into(),
                value: "Tom & Jerry".into(),
            }]
        );
    }
}
