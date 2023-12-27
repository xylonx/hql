use crate::html::ElementOrTextRef;

use super::Selector;

#[derive(Debug, Default, PartialEq)]
pub struct FlatSelector;

impl FlatSelector {
    pub fn new() -> Self {
        Self
    }
}

impl Selector for FlatSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        std::iter::once(node)
            .flat_map(|n| n.traverse_subtree())
            .collect()
    }
}

#[derive(Debug, PartialEq, Hash)]
pub enum Path {
    Single,
    Travel,
}

#[derive(Debug, PartialEq, Hash)]
pub struct PathSelector {
    paths: Vec<(Path, String)>,
}

impl PathSelector {
    pub fn new(paths: Vec<(Path, String)>) -> Self {
        Self { paths }
    }
}

impl Selector for PathSelector {
    fn select<'a, 'b: 'a>(&'b self, node: ElementOrTextRef<'a>) -> Vec<ElementOrTextRef<'a>> {
        let mut nodes = vec![node];
        for (path, tag) in &self.paths {
            nodes = match path {
                Path::Single => nodes
                    .into_iter()
                    .flat_map(|n| n.traverse_children(false))
                    .filter(|n| match n {
                        ElementOrTextRef::Element(e) => {
                            e.expanded_name().local.eq_str_ignore_ascii_case(tag)
                        }
                        _ => false,
                    })
                    .collect(),
                Path::Travel => nodes
                    .into_iter()
                    .flat_map(|n| n.traverse_subtree())
                    .filter(|n| match n {
                        ElementOrTextRef::Element(e) => {
                            e.expanded_name().local.eq_str_ignore_ascii_case(tag)
                        }

                        _ => false,
                    })
                    .collect(),
            }
        }

        nodes
    }
}
