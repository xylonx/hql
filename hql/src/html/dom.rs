use std::{
    cell::OnceCell,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
};

use html5ever::{tendril::StrTendril, Attribute, ExpandedName, LocalName, QualName};
use tracing::info;

#[derive(Debug, Clone)]
pub enum DomNode {
    Document,
    Fragment,

    DocType(DocType),
    Element(Element),
    Text(Text),
    Comment(Comment),
    ProcessingInstruction(ProcessingInstruction),
}

impl DomNode {
    pub fn is_document(&self) -> bool {
        matches!(self, DomNode::Document)
    }

    pub fn is_fragment(&self) -> bool {
        matches!(self, DomNode::Fragment)
    }

    pub fn is_doctype(&self) -> bool {
        matches!(self, DomNode::DocType(_))
    }

    pub fn is_element(&self) -> bool {
        matches!(self, DomNode::Element(_))
    }

    pub fn is_text(&self) -> bool {
        matches!(self, DomNode::Text(_))
    }

    pub fn is_comment(&self) -> bool {
        matches!(self, DomNode::Comment(_))
    }

    pub fn is_processing_instruction(&self) -> bool {
        matches!(self, DomNode::ProcessingInstruction(_))
    }

    pub fn as_doctype(&self) -> Option<&DocType> {
        match self {
            DomNode::DocType(d) => Some(d),
            _ => None,
        }
    }

    pub fn as_element(&self) -> Option<&Element> {
        match self {
            DomNode::Element(e) => Some(e),
            _ => None,
        }
    }

    pub fn as_text(&self) -> Option<&Text> {
        match self {
            DomNode::Text(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_processing_instruction(&self) -> Option<&ProcessingInstruction> {
        match self {
            DomNode::ProcessingInstruction(p) => Some(p),
            _ => None,
        }
    }
}

impl Display for DomNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DomNode::Document => write!(f, "Document"),
            DomNode::Fragment => write!(f, "Fragment"),
            DomNode::DocType(d) => write!(f, "{d}"),
            DomNode::Element(e) => write!(f, "{e}"),
            DomNode::Text(t) => write!(f, " {t}"),
            DomNode::Comment(c) => write!(f, "{c}"),
            DomNode::ProcessingInstruction(pi) => write!(f, "{pi}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DocType {
    name: StrTendril,
    public_id: StrTendril,
    system_id: StrTendril,
}

impl DocType {
    pub fn new(name: StrTendril, public_id: StrTendril, system_id: StrTendril) -> Self {
        Self {
            name,
            public_id,
            system_id,
        }
    }
}

impl Display for DocType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<!DOCTYPE {} PUBLIC {} {}>",
            self.name, self.public_id, self.system_id
        )
    }
}

#[derive(Debug, Clone)]
pub struct Element {
    name: QualName,
    attrs: HashMap<QualName, StrTendril>,

    // cache id and classes
    id: OnceCell<Option<StrTendril>>,
    classes: OnceCell<HashSet<LocalName>>,
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<{} ", self.name.local)?;
        for (k, v) in self.attrs.iter() {
            write!(f, "{}={} ", k.local, v)?;
        }
        write!(f, ">")
    }
}

impl Element {
    pub fn new(name: QualName, attrs: Vec<Attribute>) -> Self {
        Self {
            name,
            attrs: attrs.into_iter().map(|a| (a.name, a.value)).collect(),
            id: OnceCell::new(),
            classes: OnceCell::new(),
        }
    }

    pub fn expanded_name(&self) -> ExpandedName {
        self.name.expanded()
    }

    pub fn id(&self) -> Option<&str> {
        self.id
            .get_or_init(|| {
                self.attrs
                    .iter()
                    .find(|(n, _)| n.local.eq_str_ignore_ascii_case("id"))
                    .map(|(_, v)| v.clone())
            })
            .as_deref()
    }

    pub fn classes(&self) -> &HashSet<LocalName> {
        self.classes.get_or_init(|| {
            self.attrs
                .iter()
                .filter(|(n, _)| n.local.eq_str_ignore_ascii_case("class"))
                .flat_map(|(_, v)| v.split_whitespace().map(LocalName::from))
                .collect::<HashSet<LocalName>>()
        })
    }

    pub(crate) fn add_attrs(&mut self, attrs: Vec<Attribute>) {
        attrs.into_iter().for_each(|attr| {
            self.attrs.entry(attr.name).or_insert(attr.value);
        })
    }

    pub fn has_class(&self, cls: &str, case_sensitive: bool) -> bool {
        self.classes().iter().any(|c| match case_sensitive {
            true => c == cls,
            false => c.eq_str_ignore_ascii_case(cls),
        })
    }

    pub fn get_attrs(&self, name: &QualName) -> Option<&StrTendril> {
        info!("attrs: {:?}", self.attrs);
        self.attrs.get(name)
    }
}

#[derive(Debug, Clone)]
pub struct Text {
    text: StrTendril,
}

impl Text {
    pub fn new(text: StrTendril) -> Self {
        Self { text }
    }

    pub fn push_tendril(&mut self, s: &StrTendril) {
        self.text.push_tendril(s)
    }

    pub fn text(&self) -> &StrTendril {
        &self.text
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

#[derive(Debug, Clone)]
pub struct Comment {
    comment: StrTendril,
}

impl Comment {
    pub fn new(comment: StrTendril) -> Self {
        Self { comment }
    }
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<!-- {} -->", self.comment)
    }
}

#[derive(Debug, Clone)]
pub struct ProcessingInstruction {
    target: StrTendril,
    data: StrTendril,
}

impl ProcessingInstruction {
    pub fn new(target: StrTendril, data: StrTendril) -> Self {
        Self { target, data }
    }
}

impl Display for ProcessingInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<? {} {} ?>", self.target, self.data)
    }
}
