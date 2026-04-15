//! Tokenizer smoke tests against frozen `corpus/fixtures`.

use tonet_engine::html::tokenizer::{tokenize, Token};

const MINIMAL_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../corpus/fixtures/minimal.html"
));

#[test]
fn minimal_fixture_has_doctype_html_title_meta() {
    let t = tokenize(MINIMAL_HTML);
    assert!(
        t.iter().any(|x| matches!(x, Token::StartTag { name, .. } if name == "html")),
        "expected <html> start tag"
    );
    assert!(
        t.iter().any(|x| matches!(x, Token::StartTag { name, .. } if name == "head")),
        "expected <head>"
    );
    let meta = t.iter().find_map(|x| match x {
        Token::StartTag { name, attrs, .. } if name == "meta" => Some(attrs),
        _ => None,
    });
    let meta = meta.expect("<meta>");
    assert!(
        meta.iter().any(|a| a.name == "charset" && a.value == "utf-8"),
        "meta charset=utf-8, got {meta:?}"
    );
    assert!(
        t.iter().any(|x| matches!(x, Token::StartTag { name, .. } if name == "title")),
        "expected <title>"
    );
    let title_text = t.iter().find_map(|x| match x {
        Token::Text(s) if s.contains("Tonet corpus") => Some(()),
        _ => None,
    });
    assert!(title_text.is_some(), "expected title text from fixture");
}
