//! The HTML DOM tree
//!
//! Parse HTML as a DOM tree, using [html5ever](https://docs.rs/html5ever).
#[allow(dead_code)]
pub mod dom;
pub mod tree_sink;

use std::{borrow::Cow, fmt::Display, rc::Rc};

use html5ever::{
    driver,
    tendril::{StrTendril, TendrilSink},
    tokenizer::TokenizerOpts,
    tree_builder::{QuirksMode, TreeBuilderOpts},
    ExpandedName, ParseOpts, QualName,
};
use tracing::warn;

use crate::tree::{ChildrenTraverse, Node, PreOrderTraverse, Tree};

use self::dom::{DomNode, Text};

#[derive(Debug)]
pub struct Html {
    nodes: Tree<DomNode>,

    quirks_mode: QuirksMode,

    errors: Vec<Cow<'static, str>>,
}

impl Html {
    fn new_document() -> Self {
        Self {
            nodes: Tree::new(DomNode::Document),
            quirks_mode: QuirksMode::NoQuirks,
            errors: vec![],
        }
    }

    fn new_fragment() -> Self {
        Self {
            nodes: Tree::new(DomNode::Fragment),
            quirks_mode: QuirksMode::NoQuirks,
            errors: Vec::new(),
        }
    }

    pub fn parse_document(doc: &str, exact_errors: bool) -> Self {
        driver::parse_document(
            Self::new_document(),
            ParseOpts {
                tokenizer: TokenizerOpts {
                    exact_errors,
                    ..TokenizerOpts::default()
                },
                tree_builder: TreeBuilderOpts {
                    exact_errors,
                    ..TreeBuilderOpts::default()
                },
            },
        )
        .one(doc)
    }

    pub fn parse_fragment(frag: &str, exact_errors: bool) -> Self {
        driver::parse_fragment(
            Self::new_fragment(),
            ParseOpts {
                tokenizer: TokenizerOpts {
                    exact_errors,
                    ..TokenizerOpts::default()
                },
                tree_builder: TreeBuilderOpts {
                    exact_errors,
                    ..TreeBuilderOpts::default()
                },
            },
            QualName::new(None, ns!(html), local_name!("body")),
            Vec::new(),
        )
        .one(frag)
    }
}

impl Html {
    pub fn root(&self) -> ElementOrTextRef {
        ElementOrTextRef::Element(ElementRef {
            node: self.nodes.root_ref().unwrap(),
            tree: &self.nodes,
        })
    }

    pub fn traverse_all(&self) -> Vec<DomNode> {
        PreOrderTraverse::new(&self.nodes, self.nodes.root_ref().unwrap())
            .map(move |(n, _)| n.data.clone())
            .collect()
    }
}

impl Display for Html {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for n in self.nodes.nodes() {
            write!(f, "{}", n.data)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ElementRef<'a> {
    tree: &'a Tree<DomNode>,
    node: &'a Node<DomNode>,
}

impl<'a> Display for ElementRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.node)
    }
}

impl<'a> ElementRef<'a> {
    pub fn expanded_name(&self) -> ExpandedName {
        self.node.data.as_element().unwrap().expanded_name()
    }

    pub fn get_attr(&self, name: &QualName) -> Option<&StrTendril> {
        self.node.data.as_element().unwrap().get_attrs(name)
    }

    pub fn has_class(&self, class: &str, case_sensitive: bool) -> bool {
        self.node
            .data
            .as_element()
            .unwrap()
            .has_class(class, case_sensitive)
    }

    pub fn has_id(&self, id: &str, case_sensitive: bool) -> bool {
        self.node
            .data
            .as_element()
            .unwrap()
            .id()
            .map(|i| match case_sensitive {
                true => i == id,
                false => i.eq_ignore_ascii_case(id),
            })
            .is_some()
    }

    // For element, traverse the whole subtree and extract its text
    pub fn text(&self) -> impl Iterator<Item = &Text> {
        PreOrderTraverse::new(self.tree, self.node).filter_map(|(n, _)| match &n.data {
            DomNode::Text(t) => Some(t),
            _ => None,
        })
    }

    pub fn children(self, reversed: bool) -> impl Iterator<Item = ElementOrTextRef<'a>> {
        ChildrenTraverse::new(self.tree, self.node, reversed).filter_map(|(n, t)| match n.data {
            DomNode::Element(_) => Some(ElementOrTextRef::Element(ElementRef { tree: t, node: n })),
            DomNode::Text(_) => Some(ElementOrTextRef::Text(TextRef { tree: t, node: n })),
            _ => None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TextRef<'a> {
    tree: &'a Tree<DomNode>,
    node: &'a Node<DomNode>,
}

impl<'a> TextRef<'a> {
    pub fn text(&self) -> &Text {
        self.node.data.as_text().unwrap()
    }
}

impl<'a> Display for TextRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.node)
    }
}

#[derive(Debug, Clone)]
pub struct PhantomTextRef {
    text: Rc<Node<DomNode>>,
}

impl PhantomTextRef {
    pub fn new(text: Text) -> Self {
        Self {
            text: Rc::new(Node::phantom(DomNode::Text(text))),
        }
    }

    pub fn new_with_txt(txt: StrTendril) -> Self {
        Self {
            text: Rc::new(Node::phantom(DomNode::Text(Text::new(txt)))),
        }
    }

    pub fn new_with<F>(mut f: F) -> Self
    where
        F: FnMut() -> Text,
    {
        Self {
            text: Rc::new(Node::phantom(DomNode::Text(f()))),
        }
    }

    pub fn text(&self) -> &Text {
        self.text.data.as_text().unwrap()
    }
}

impl Display for PhantomTextRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug, Clone)]
pub enum ElementOrTextRef<'a> {
    Element(ElementRef<'a>),
    Text(TextRef<'a>),
    PhantomText(PhantomTextRef),
}

impl<'a> Display for ElementOrTextRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ElementOrTextRef::Element(e) => write!(f, "{}", e),
            ElementOrTextRef::Text(t) => write!(f, "{}", t),
            ElementOrTextRef::PhantomText(t) => write!(f, "{}", t),
        }
    }
}
impl<'a> ElementOrTextRef<'a> {
    fn into_children(self, reversed: bool) -> Option<ChildrenTraverse<'a, DomNode>> {
        match self {
            ElementOrTextRef::Element(e) => Some(ChildrenTraverse::new(e.tree, e.node, reversed)),
            ElementOrTextRef::Text(t) => Some(ChildrenTraverse::new(t.tree, t.node, reversed)),
            ElementOrTextRef::PhantomText(_) => None,
        }
    }

    pub fn new_phantom_from_text(text: Text) -> Self {
        Self::PhantomText(PhantomTextRef::new(text))
    }

    pub fn new_phantom_from_txt(txt: StrTendril) -> Self {
        Self::PhantomText(PhantomTextRef::new_with_txt(txt))
    }
}

impl<'a> From<ElementOrTextRef<'a>> for Option<PreOrderTraverse<'a, DomNode>> {
    fn from(val: ElementOrTextRef<'a>) -> Self {
        match val {
            ElementOrTextRef::Element(e) => Some(PreOrderTraverse::new(e.tree, e.node)),
            ElementOrTextRef::Text(t) => Some(PreOrderTraverse::new(t.tree, t.node)),
            ElementOrTextRef::PhantomText(_) => None,
        }
    }
}

impl<'a> ElementOrTextRef<'a> {
    pub fn node(&self) -> &Node<DomNode> {
        match self {
            ElementOrTextRef::Element(e) => e.node,
            ElementOrTextRef::Text(t) => t.node,
            ElementOrTextRef::PhantomText(t) => &t.text,
        }
    }

    pub fn traverse_subtree(self) -> impl Iterator<Item = ElementOrTextRef<'a>> + 'a {
        Into::<Option<PreOrderTraverse<'a, DomNode>>>::into(self)
            .map(|t| {
                t.filter_map(|(node, tree)| match &node.data {
                    DomNode::Element(_) => {
                        Some(ElementOrTextRef::Element(ElementRef { node, tree }))
                    }
                    DomNode::Text(_) => Some(ElementOrTextRef::Text(TextRef { node, tree })),
                    e => {
                        warn!("unsupported dom node: {}", e);
                        None
                    }
                })
            })
            .into_iter()
            .flatten()
    }

    pub fn traverse_children(
        self,
        reversed: bool,
    ) -> impl Iterator<Item = ElementOrTextRef<'a>> + 'a {
        self.into_children(reversed)
            .map(|t| {
                t.filter_map(|(node, tree)| match node.data {
                    DomNode::Element(_) => {
                        Some(ElementOrTextRef::Element(ElementRef { node, tree }))
                    }
                    DomNode::Text(_) => Some(ElementOrTextRef::Text(TextRef { node, tree })),
                    _ => None,
                })
            })
            .into_iter()
            .flatten()
    }
}

#[cfg(test)]
mod test {
    use std::fs;

    use tracing::level_filters::LevelFilter;

    use super::Html;

    #[test]
    fn test_parse_document() {
        tracing_subscriber::fmt::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::builder()
                    .with_default_directive(LevelFilter::INFO.into())
                    .from_env_lossy(),
            )
            .init();

        let s = fs::read_to_string("./sof.html").unwrap();

        let dom = Html::parse_document(&s, true);

        // println!("{}", dom);
        dom.traverse_all()
            .into_iter()
            .for_each(|n| println!("{}", n));
    }
}
