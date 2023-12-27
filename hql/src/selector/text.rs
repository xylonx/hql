use std::str::FromStr;

use html5ever::tendril::StrTendril;

use crate::html::ElementOrTextRef;

use super::Selector;

#[derive(Debug, Default, PartialEq)]
pub struct TextSelector;

impl TextSelector {
    pub fn new() -> Self {
        TextSelector
    }
}

impl Selector for TextSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .map(|n| match n {
                ElementOrTextRef::Element(e) => {
                    ElementOrTextRef::new_phantom_from_txt(e.text().map(|t| t.text()).collect())
                }
                _ => n,
            })
            .collect()
    }
}

/// TrimSelector will only handle Text and PhantomText nodes and ignore element nodes
#[derive(Debug, Default, PartialEq)]
pub struct TrimSelector;

impl TrimSelector {
    pub fn new() -> Self {
        Self
    }
}

impl Selector for TrimSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .map(|n| match n {
                ElementOrTextRef::Element(_) => n,
                ElementOrTextRef::Text(t) => ElementOrTextRef::new_phantom_from_txt(
                    StrTendril::from_str(t.text().text().clone().trim()).unwrap(),
                ),
                ElementOrTextRef::PhantomText(t) => ElementOrTextRef::new_phantom_from_txt(
                    StrTendril::from_str(t.text().text().clone().trim()).unwrap(),
                ),
            })
            .collect()
    }
}

/// TrimPrefixSelector will only handle Text and PhantomText nodes and ignore element nodes
#[derive(Debug, PartialEq)]
pub struct TrimPrefixSelector {
    prefix: String,
}

impl TrimPrefixSelector {
    pub fn new(prefix: String) -> Self {
        Self { prefix }
    }
}

impl Selector for TrimPrefixSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .map(|n| match n {
                ElementOrTextRef::Element(_) => n,
                ElementOrTextRef::Text(t) => {
                    let t = t.text().text().clone();
                    let striped = t.strip_prefix(&self.prefix).unwrap_or(&t);
                    ElementOrTextRef::new_phantom_from_txt(StrTendril::from_str(striped).unwrap())
                }
                ElementOrTextRef::PhantomText(t) => {
                    let t = t.text().text().clone();
                    let striped = t.strip_prefix(&self.prefix).unwrap_or(&t);
                    ElementOrTextRef::new_phantom_from_txt(StrTendril::from_str(striped).unwrap())
                }
            })
            .collect()
    }
}

/// TrimSuffixSelector will only handle Text and PhantomText nodes and ignore element nodes
#[derive(Debug, PartialEq)]
pub struct TrimSuffixSelector {
    suffix: String,
}

impl TrimSuffixSelector {
    pub fn new(suffix: String) -> Self {
        Self { suffix }
    }
}

impl Selector for TrimSuffixSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .map(|n| match n {
                ElementOrTextRef::Element(_) => n,
                ElementOrTextRef::Text(t) => {
                    let t = t.text().text().clone();
                    let striped = t.strip_suffix(&self.suffix).unwrap_or(&t);
                    ElementOrTextRef::new_phantom_from_txt(StrTendril::from_str(striped).unwrap())
                }
                ElementOrTextRef::PhantomText(t) => {
                    let t = t.text().text().clone();
                    let striped = t.strip_suffix(&self.suffix).unwrap_or(&t);
                    ElementOrTextRef::new_phantom_from_txt(StrTendril::from_str(striped).unwrap())
                }
            })
            .collect()
    }
}

/// NthChildSelector will filter out Text nodes, PhantomText nodes and Element nodes without sufficient children
#[derive(Debug, PartialEq)]
pub struct NthChildSelector {
    n: usize,
    reversed: bool,
}

impl NthChildSelector {
    pub fn new(n: usize, reversed: bool) -> Self {
        Self { n, reversed }
    }
}

impl Selector for NthChildSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .filter_map(|n| match n {
                ElementOrTextRef::Element(e) => e.children(self.reversed).nth(self.n),
                _ => None,
            })
            .collect()
    }
}
