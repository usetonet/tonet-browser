//! Build a **DOM-shaped tree** from tokenizer output.
//!
//! This is not the full HTML5 tree-construction algorithm (no foster parenting, adoption, etc.);
//! it stacks open elements and closes on matching `</tag>`, with light recovery on EOF.

use super::attributes::Attr;
use super::tokenizer::Token;

/// Document fragment: top-level nodes (typically one `<html>` in a full document).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFragment {
    pub children: Vec<Node>,
}

/// A node in the simplified tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Element(ElementNode),
    /// Character data (entities not decoded yet).
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementNode {
    pub name: String,
    pub attrs: Vec<Attr>,
    pub children: Vec<Node>,
}

/// HTML **void** elements — never retain children in the DOM; closing tags are optional.
fn is_void_element(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

/// Build a tree from a token stream (usually from [`super::tokenizer::tokenize`]).
/// Comments are dropped. Text outside any element is attached to the fragment root.
pub fn build_fragment(tokens: &[Token]) -> DocumentFragment {
    let mut roots: Vec<Node> = Vec::new();
    let mut stack: Vec<ElementNode> = Vec::new();

    fn append_text(stack: &mut Vec<ElementNode>, roots: &mut Vec<Node>, text: String) {
        if text.is_empty() {
            return;
        }
        let n = Node::Text(text);
        if let Some(parent) = stack.last_mut() {
            parent.children.push(n);
        } else {
            roots.push(n);
        }
    }

    for t in tokens {
        match t {
            Token::Text(s) => {
                append_text(&mut stack, &mut roots, s.clone());
            }
            Token::StartTag {
                name,
                self_closing,
                attrs,
            } => {
                let el = ElementNode {
                    name: name.clone(),
                    attrs: attrs.clone(),
                    children: vec![],
                };
                if *self_closing || is_void_element(name) {
                    let node = Node::Element(el);
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(node);
                    } else {
                        roots.push(node);
                    }
                } else {
                    stack.push(el);
                }
            }
            Token::EndTag { name } => {
                close_until_name(&mut stack, &mut roots, name);
            }
            Token::Comment(_) => {}
            Token::EndOfFile => break,
        }
    }

    // Implicitly close any still-open elements (EOF recovery).
    while let Some(el) = stack.pop() {
        let node = Node::Element(el);
        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        } else {
            roots.push(node);
        }
    }

    DocumentFragment { children: roots }
}

/// Pop until `name` matches the top of stack, then attach the closed subtree to its parent.
fn close_until_name(stack: &mut Vec<ElementNode>, roots: &mut Vec<Node>, name: &str) {
    let name = name.trim();
    if name.is_empty() {
        return;
    }

    // Find matching open element (search from top); auto-close mismatched inner tags first.
    let mut idx = None;
    for (i, el) in stack.iter().enumerate().rev() {
        if el.name == name {
            idx = Some(i);
            break;
        }
    }

    let Some(idx) = idx else {
        return;
    };

    while stack.len() > idx + 1 {
        let orphan = stack.pop().unwrap();
        let node = Node::Element(orphan);
        if let Some(parent) = stack.last_mut() {
            parent.children.push(node);
        } else {
            roots.push(node);
        }
    }

    let closed = stack.pop().unwrap();
    let node = Node::Element(closed);
    if let Some(parent) = stack.last_mut() {
        parent.children.push(node);
    } else {
        roots.push(node);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html::tokenizer::tokenize;

    #[test]
    fn paragraph_with_text() {
        let tokens = tokenize("<p>hello</p>");
        let doc = build_fragment(&tokens);
        assert_eq!(doc.children.len(), 1);
        let Node::Element(p) = &doc.children[0] else {
            panic!("expected p");
        };
        assert_eq!(p.name, "p");
        assert_eq!(p.children.len(), 1);
        assert!(matches!(&p.children[0], Node::Text(t) if t == "hello"));
    }

    #[test]
    fn nested_elements() {
        let tokens = tokenize("<div><span>a</span></div>");
        let doc = build_fragment(&tokens);
        let Node::Element(div) = &doc.children[0] else {
            panic!();
        };
        assert_eq!(div.children.len(), 1);
        let Node::Element(span) = &div.children[0] else {
            panic!();
        };
        assert_eq!(span.name, "span");
        assert!(matches!(&span.children[0], Node::Text(t) if t == "a"));
    }

    #[test]
    fn void_meta_no_children() {
        let tokens = tokenize(r#"<meta charset="utf-8">"#);
        let doc = build_fragment(&tokens);
        let Node::Element(m) = &doc.children[0] else {
            panic!();
        };
        assert_eq!(m.name, "meta");
        assert!(m.children.is_empty());
        assert!(m.attrs.iter().any(|a| a.name == "charset"));
    }

    #[test]
    fn br_self_closing_inside_p() {
        let tokens = tokenize("<p>a<br/>b</p>");
        let doc = build_fragment(&tokens);
        let Node::Element(p) = &doc.children[0] else {
            panic!();
        };
        assert_eq!(p.children.len(), 3);
        assert!(matches!(&p.children[0], Node::Text(t) if t == "a"));
        let Node::Element(br) = &p.children[1] else {
            panic!();
        };
        assert_eq!(br.name, "br");
        assert!(matches!(&p.children[2], Node::Text(t) if t == "b"));
    }
}
