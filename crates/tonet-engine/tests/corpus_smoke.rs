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

const WITH_LINKS_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../corpus/fixtures/with_links.html"
));

#[test]
fn with_links_fixture_has_anchor_hrefs() {
    let lower = WITH_LINKS_HTML.to_ascii_lowercase();
    assert!(
        lower.contains("href=\"https://example.com/a\""),
        "with_links should include first external href"
    );
    assert!(
        lower.contains("href=\"https://example.com/b\""),
        "with_links should include second external href"
    );
}
