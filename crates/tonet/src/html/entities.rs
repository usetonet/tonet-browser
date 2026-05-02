//! HTML **character reference** decoding (subset of WHATWG “named character references”).
//!
//! Supports common named entities, decimal `&#…;`, and hexadecimal `&#x…;` / `&#X…;`.
//! Unknown or malformed references are left as literal text (starting with `&`).

/// Maximum length of `&…;` body (excluding `&` and `;`) for DoS safety.
const MAX_ENTITY_BODY_LEN: usize = 64;

/// Decode all well-formed `&…;` sequences in `input`. Pass-through when no `&` or no match.
pub fn decode_html_entities(input: &str) -> String {
    if !input.contains('&') {
        return input.to_string();
    }
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    let b = input.as_bytes();
    while i < input.len() {
        if b.get(i) == Some(&b'&') {
            if let Some((ch, len)) = try_decode_entity_at(&input[i..]) {
                out.push(ch);
                i += len;
                continue;
            }
        }
        let c = input[i..].chars().next().unwrap();
        out.push(c);
        i += c.len_utf8();
    }
    out
}

/// If `s` starts with `&` and a well-formed reference, returns `(decoded, consumed_bytes)`.
fn try_decode_entity_at(s: &str) -> Option<(char, usize)> {
    if !s.starts_with('&') {
        return None;
    }
    let rest = s.get(1..)?;
    let semi = rest.find(';')?;
    if semi > MAX_ENTITY_BODY_LEN {
        return None;
    }
    let body = &rest[..semi];
    let ch = resolve_entity(body)?;
    Some((ch, 1 + semi + 1))
}

fn resolve_entity(body: &str) -> Option<char> {
    match body {
        "amp" => return Some('&'),
        "lt" => return Some('<'),
        "gt" => return Some('>'),
        "quot" => return Some('"'),
        "apos" => return Some('\''),
        "nbsp" => return Some('\u{00A0}'),
        _ => {}
    }

    let bytes = body.as_bytes();
    if bytes.first() == Some(&b'#') && bytes.len() >= 3 {
        if bytes[1] == b'x' || bytes[1] == b'X' {
            let hex = std::str::from_utf8(&bytes[2..]).ok()?;
            let n = u32::from_str_radix(hex, 16).ok()?;
            return char::from_u32(n);
        }
    }

    if let Some(dec) = body.strip_prefix('#') {
        if !dec.is_empty() && dec.chars().all(|c| c.is_ascii_digit()) {
            let n = u32::from_str_radix(dec, 10).ok()?;
            return char::from_u32(n);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn amp_lt_gt() {
        assert_eq!(
            decode_html_entities("a &amp; b &lt; c &gt;"),
            "a & b < c >"
        );
    }

    #[test]
    fn decimal_and_hex() {
        assert_eq!(decode_html_entities("&#39;"), "'");
        assert_eq!(decode_html_entities("&#x27;"), "'");
        assert_eq!(decode_html_entities("&#X41;"), "A");
    }

    #[test]
    fn nbsp() {
        assert_eq!(decode_html_entities("a&nbsp;b").chars().nth(1), Some('\u{00A0}'));
    }

    #[test]
    fn unknown_passthrough() {
        assert_eq!(decode_html_entities("&nope;"), "&nope;");
    }

    #[test]
    fn incomplete_no_semicolon() {
        assert_eq!(decode_html_entities("&amp"), "&amp");
    }
}
