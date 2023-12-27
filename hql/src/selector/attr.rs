use std::str::FromStr;

use html5ever::{tendril::StrTendril, LocalName, QualName};

use crate::html::ElementOrTextRef;

use super::Selector;

///
#[derive(Debug, PartialEq)]
pub struct AttrSelector {
    name: QualName,
    /// val: none means filter whether attr:name exists
    val: Option<StrTendril>,
}

impl AttrSelector {
    pub fn new(name: &str, val: Option<&str>) -> Self {
        Self {
            name: QualName::new(None, ns!(), LocalName::from(name)),
            val: val.map(|v| StrTendril::from_str(v).unwrap()),
        }
    }
}

impl Selector for AttrSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .filter(|n| match n {
                ElementOrTextRef::Element(e) => {
                    e.get_attr(&self.name).iter().any(|s| match &self.val {
                        None => true,
                        Some(v) => s.eq_ignore_ascii_case(v),
                    })
                }
                _ => false,
            })
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub struct ClassSelector {
    class: String,
    case_sensitive: bool,
}

impl ClassSelector {
    pub fn new(class: String, case_sensitive: bool) -> Self {
        Self {
            class,
            case_sensitive,
        }
    }
}

impl Selector for ClassSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .filter(|n| match n {
                ElementOrTextRef::Element(e) => e.has_class(&self.class, self.case_sensitive),
                _ => false,
            })
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub struct IDSelector {
    id: String,
    case_sensitive: bool,
}

impl IDSelector {
    pub fn new(id: String, case_sensitive: bool) -> Self {
        Self { id, case_sensitive }
    }
}

impl Selector for IDSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .filter(|n| match n {
                ElementOrTextRef::Element(e) => e.has_id(&self.id, self.case_sensitive),
                _ => false,
            })
            .collect()
    }
}

#[derive(Debug, PartialEq)]
pub struct ExtractAttrSelector {
    attr: QualName,
}

impl ExtractAttrSelector {
    pub fn new(attr: &str) -> Self {
        Self {
            attr: QualName::new(None, ns!(), LocalName::from(attr)),
        }
    }
}

impl Selector for ExtractAttrSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .filter_map(|n| match n {
                ElementOrTextRef::Element(e) => e
                    .get_attr(&self.attr)
                    .map(|txt| ElementOrTextRef::new_phantom_from_txt(txt.clone())),
                _ => None,
            })
            .collect()
    }
}
