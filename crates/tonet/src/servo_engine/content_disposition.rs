//! Best-effort `Content-Disposition` **filename** extraction for save-as flows (see `background_download`).

#[inline]
fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(10 + (b - b'a')),
        b'A'..=b'F' => Some(10 + (b - b'A')),
        _ => None,
    }
}

/// Decode `%HH` sequences; invalid escapes leave the `%` as a literal byte (lossy UTF-8).
fn percent_decode_lossy(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = Vec::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let (Some(h1), Some(h2)) = (hex_val(b[i + 1]), hex_val(b[i + 2])) {
                out.push(h1 << 4 | h2);
                i += 3;
                continue;
            }
        }
        out.push(b[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[inline]
fn finalize_filename(raw: &str) -> Option<String> {
    let t = raw.trim();
    if t.is_empty() {
        return None;
    }
    Some(percent_decode_lossy(t))
}

/// `true` when byte at `idx` is preceded by an **odd** number of consecutive ASCII `\` bytes (so
/// `\"` starts an escaped quote, `\\"` does not).
#[inline]
fn is_backslash_escaped_dquote(bytes: &[u8], idx: usize) -> bool {
    let mut n = 0usize;
    let mut j = idx;
    while j > 0 && bytes[j - 1] == b'\\' {
        n += 1;
        j -= 1;
    }
    n % 2 == 1
}

/// Content of a quoted-string: `s` is the slice **after** the opening `"`; returns bytes before
/// the first unescaped closing `"`.
fn slice_before_unescaped_dquote(s: &str) -> Option<&str> {
    let bytes = s.as_bytes();
    for (i, ch) in s.char_indices() {
        if ch == '"' && !is_backslash_escaped_dquote(bytes, i) {
            return Some(&s[..i]);
        }
    }
    None
}

/// HTTP-ish quoted-string unescape for **`filename="..."` only**: `\` + `"` → `"`, `\` + `\` → `\`;
/// lone `\` at end is kept.
fn unescape_quoted_filename_body(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some('"') => out.push('"'),
            Some('\\') => out.push('\\'),
            Some(x) => {
                out.push('\\');
                out.push(x);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Split `Content-Disposition` on `;` **outside** ASCII double-quoted spans (toggle on `"` unless
/// the `"` is backslash-escaped). Unquoted `filename*=` values should not embed raw `;` (use RFC
/// 5987 `%3B`); otherwise the payload is truncated at the first `;`.
fn disposition_params(cd: &str) -> Vec<&str> {
    let bytes = cd.as_bytes();
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut in_dq = false;
    for (idx, c) in cd.char_indices() {
        match c {
            '"' => {
                if is_backslash_escaped_dquote(bytes, idx) {
                    continue;
                }
                in_dq = !in_dq;
            }
            ';' if !in_dq => {
                let s = cd[start..idx].trim();
                if !s.is_empty() {
                    out.push(s);
                }
                start = idx + c.len_utf8();
            }
            _ => {}
        }
    }
    let tail = cd[start..].trim();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

/// Returns the filename from a full `Content-Disposition` header value (no path validation).
/// Parses parameters on **`;` boundaries outside ASCII double-quoted spans** (so `filename="a;b"` works;
/// `\"` does not end a span). Closing **`filename="..."`** uses the same backslash rule.
/// Tokens like `notfilename=` do not match. Supports
/// `filename="..."`, unquoted `filename=...`, and `filename*=...''...` (payload after `''`, with
/// **light** `%HH` decoding). Quoted `filename=` bodies apply **`\\` / `\"` unescaping** (then `%HH`).
/// `filename*=` wins over plain `filename=` when the star form yields a
/// non-empty value after `''`. If `filename*=` has no `''` or an empty payload, plain `filename=`
/// is considered. A leading **UTF-8 BOM** (`U+FEFF`) after optional ASCII whitespace is ignored.
pub(crate) fn parse_filename_value(cd: &str) -> Option<String> {
    const FILENAME_STAR: &[u8] = b"filename*=";
    const FILENAME: &[u8] = b"filename=";

    let cd = cd.trim_start();
    let cd = cd.strip_prefix('\u{FEFF}').unwrap_or(cd);

    let parts = disposition_params(cd);

    for part in &parts {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let b = p.as_bytes();
        if b.len() < FILENAME_STAR.len()
            || !b[..FILENAME_STAR.len()].eq_ignore_ascii_case(FILENAME_STAR)
        {
            continue;
        }
        let rest = &p[FILENAME_STAR.len()..];
        if let Some(apo) = rest.find("''") {
            let val = rest[apo + 2..].trim().trim_matches(';').trim();
            let val = val.trim_matches('"');
            if let Some(s) = finalize_filename(val) {
                return Some(s);
            }
        }
    }

    for part in &parts {
        let p = part.trim();
        if p.is_empty() {
            continue;
        }
        let b = p.as_bytes();
        if b.len() >= FILENAME_STAR.len()
            && b[..FILENAME_STAR.len()].eq_ignore_ascii_case(FILENAME_STAR)
        {
            continue;
        }
        if b.len() < FILENAME.len() || !b[..FILENAME.len()].eq_ignore_ascii_case(FILENAME) {
            continue;
        }
        let rest = &p[FILENAME.len()..].trim_start();
        if let Some(rest) = rest.strip_prefix('"') {
            if let Some(body) = slice_before_unescaped_dquote(rest) {
                let s = unescape_quoted_filename_body(body.trim());
                if let Some(s) = finalize_filename(&s) {
                    return Some(s);
                }
            }
        } else {
            let s = rest.trim();
            if let Some(s) = finalize_filename(s) {
                return Some(s);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quoted_filename() {
        assert_eq!(
            parse_filename_value(r#"attachment; filename="hello.zip""#).as_deref(),
            Some("hello.zip")
        );
    }

    #[test]
    fn unquoted_filename_until_semicolon() {
        assert_eq!(
            parse_filename_value("attachment; filename=report.pdf; size=123").as_deref(),
            Some("report.pdf")
        );
    }

    #[test]
    fn filename_star_utf8_suffix() {
        assert_eq!(
            parse_filename_value("attachment; filename*=UTF-8''archive.tgz").as_deref(),
            Some("archive.tgz")
        );
    }

    #[test]
    fn no_filename_returns_none() {
        assert!(parse_filename_value("inline").is_none());
    }

    #[test]
    fn filename_star_takes_precedence_over_plain_when_both_present() {
        let cd = r#"attachment; filename="old.bin"; filename*=UTF-8''new.bin"#;
        assert_eq!(parse_filename_value(cd).as_deref(), Some("new.bin"));
    }

    #[test]
    fn filename_star_percent_decodes_space() {
        assert_eq!(
            parse_filename_value("attachment; filename*=UTF-8''my%20file.pdf").as_deref(),
            Some("my file.pdf")
        );
    }

    #[test]
    fn quoted_filename_percent_encoded() {
        assert_eq!(
            parse_filename_value(r#"attachment; filename="a%20b.zip""#).as_deref(),
            Some("a b.zip")
        );
    }

    #[test]
    fn percent_invalid_escape_passthrough() {
        assert_eq!(
            parse_filename_value(r#"attachment; filename="a%ZZb.zip""#).as_deref(),
            Some("a%ZZb.zip")
        );
    }

    #[test]
    fn percent_trailing_incomplete_passthrough() {
        assert_eq!(
            parse_filename_value(r#"attachment; filename="doc%.zip""#).as_deref(),
            Some("doc%.zip")
        );
    }

    #[test]
    fn quoted_filename_without_closing_quote_returns_none() {
        assert!(parse_filename_value(r#"attachment; filename="never_closes.zip"#).is_none());
    }

    #[test]
    fn filename_star_without_utf8_delimiter_falls_back_to_plain_filename() {
        let cd = r#"attachment; filename*=UTF-8-broken; filename="legacy.zip""#;
        assert_eq!(parse_filename_value(cd).as_deref(), Some("legacy.zip"));
    }

    #[test]
    fn inline_disposition_with_filename() {
        assert_eq!(
            parse_filename_value(r#"inline; filename="page.bin""#).as_deref(),
            Some("page.bin")
        );
    }

    #[test]
    fn filename_star_only_without_delimiter_returns_none() {
        assert!(parse_filename_value("attachment; filename*=UTF-8-nodelim").is_none());
    }

    #[test]
    fn notfilename_param_does_not_trigger() {
        let cd = "attachment; notfilename=trap; filename=real.zip";
        assert_eq!(parse_filename_value(cd).as_deref(), Some("real.zip"));
    }

    #[test]
    fn empty_filename_star_payload_falls_back_to_plain() {
        let cd = r#"attachment; filename*=UTF-8''; filename="plain.pdf""#;
        assert_eq!(parse_filename_value(cd).as_deref(), Some("plain.pdf"));
    }

    #[test]
    fn quoted_filename_semicolon_inside_quotes() {
        let cd = r#"attachment; filename="semi;colon.zip"; size=123"#;
        assert_eq!(parse_filename_value(cd).as_deref(), Some("semi;colon.zip"));
    }

    #[test]
    fn quoted_filename_backslash_escaped_quote_and_semicolon() {
        // `r###"..."###` so the closing `"###` does not eat the filename's final `"` (see `r##` note in Rust ref).
        let cd = r###"attachment; filename="part1\";part2.zip""###;
        assert_eq!(
            parse_filename_value(cd).as_deref(),
            Some(r#"part1";part2.zip"#)
        );
    }

    #[test]
    fn quoted_filename_doubled_backslash_collapses() {
        let cd = r#"attachment; filename="a\\b.zip""#;
        assert_eq!(parse_filename_value(cd).as_deref(), Some(r"a\b.zip"));
    }

    #[test]
    fn leading_utf8_bom_after_whitespace_stripped() {
        let cd = format!("  \u{FEFF}attachment; filename=\"bom.zip\"");
        assert_eq!(parse_filename_value(&cd).as_deref(), Some("bom.zip"));
    }

    #[test]
    fn filename_param_token_is_ascii_case_insensitive() {
        assert_eq!(
            parse_filename_value(r#"attachment; FileName="Mixed.zip""#).as_deref(),
            Some("Mixed.zip")
        );
    }

    #[test]
    fn filename_star_token_is_ascii_case_insensitive() {
        assert_eq!(
            parse_filename_value("attachment; FILENAME*=UTF-8''star.bin").as_deref(),
            Some("star.bin")
        );
    }

    #[test]
    fn empty_quoted_filename_returns_none() {
        assert!(parse_filename_value(r#"attachment; filename="""#).is_none());
    }

    #[test]
    fn filename_after_non_filename_params() {
        assert_eq!(
            parse_filename_value("attachment; size=999; filename=last.pdf").as_deref(),
            Some("last.pdf")
        );
    }

    #[test]
    fn unquoted_filename_with_internal_spaces() {
        assert_eq!(
            parse_filename_value("attachment; filename=my report.pdf").as_deref(),
            Some("my report.pdf")
        );
    }

    #[test]
    fn filename_star_percent_decodes_utf8_non_ascii() {
        assert_eq!(
            parse_filename_value("attachment; filename*=UTF-8''%c3%a4.bin").as_deref(),
            Some("ä.bin")
        );
    }
}
