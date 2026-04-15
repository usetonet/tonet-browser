//! Tree builder smoke test on `corpus/fixtures/minimal.html`.

use tonet_engine::html::tokenizer::tokenize;
use tonet_engine::html::tree_builder::build_fragment;
use tonet_engine::html::Node;

const MINIMAL_HTML: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../corpus/fixtures/minimal.html"
));

fn find_first_element<'a>(nodes: &'a [Node], name: &str) -> Option<&'a tonet_engine::html::ElementNode> {
    nodes.iter().find_map(|n| match n {
        Node::Element(e) if e.name == name => Some(e),
        _ => None,
    })
}

#[test]
fn minimal_fixture_tree_html_head_body() {
    let tokens = tokenize(MINIMAL_HTML);
    let doc = build_fragment(&tokens);
    let html = find_first_element(&doc.children, "html").expect("<html>");
    assert!(
        html.attrs.iter().any(|a| a.name == "lang" && a.value == "en"),
        "html lang"
    );

    let head = find_first_element(&html.children, "head").expect("<head>");
    let meta = find_first_element(&head.children, "meta").expect("<meta>");
    assert!(
        meta.attrs.iter().any(|a| a.name == "charset" && a.value == "utf-8"),
        "charset"
    );

    let title = find_first_element(&head.children, "title").expect("<title>");
    let title_text = title.children.iter().find_map(|n| match n {
        Node::Text(t) => Some(t.as_str()),
        _ => None,
    });
    assert!(
        title_text.is_some_and(|t| t.contains("Tonet corpus")),
        "title text"
    );

    let body = find_first_element(&html.children, "body").expect("<body>");
    let p = find_first_element(&body.children, "p").expect("<p>");
    assert!(
        p.children.iter().any(|n| matches!(n, Node::Text(t) if t.contains("Minimal frozen"))),
        "body paragraph"
    );
}
