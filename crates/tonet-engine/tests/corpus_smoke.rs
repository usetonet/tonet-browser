//! Smoke-test that the frozen corpus is present and UTF-8 sane (CI gate toward full corpus runs).

const MINIMAL_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../corpus/fixtures/minimal.html"
));

#[test]
fn minimal_fixture_has_doctype_and_title() {
    assert!(!MINIMAL_HTML.is_empty());
    let lower = MINIMAL_HTML.to_ascii_lowercase();
    assert!(
        lower.contains("<!doctype html"),
        "minimal fixture should declare HTML5 doctype"
    );
    assert!(
        lower.contains("<title") && lower.contains("</title>"),
        "minimal fixture should include a title element"
    );
    assert!(
        lower.contains("tonet corpus"),
        "minimal fixture should keep its identifying title text"
    );
}
