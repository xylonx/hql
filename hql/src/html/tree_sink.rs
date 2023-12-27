use html5ever::{
    expanded_name,
    tree_builder::{ElementFlags, NodeOrText, TreeSink},
    QualName,
};
use tracing::error;

use crate::{
    html::dom::{DomNode, Element},
    tree::NodeID,
};

use super::{
    dom::{Comment, DocType, ProcessingInstruction, Text},
    Html,
};

impl TreeSink for Html {
    type Handle = NodeID;

    type Output = Self;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&mut self, msg: std::borrow::Cow<'static, str>) {
        error!("Error occur when parsing html: {}", msg);
        self.errors.push(msg);
    }

    fn get_document(&mut self) -> Self::Handle {
        self.nodes.root_ref().unwrap().id
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> html5ever::ExpandedName<'a> {
        self.nodes
            .node_ref(*target)
            .unwrap()
            .data
            .as_element()
            .unwrap()
            .expanded_name()
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<html5ever::Attribute>,
        _flags: ElementFlags,
    ) -> Self::Handle {
        let is_fragment = name.expanded() == expanded_name!(html "template");

        let node_id = self
            .nodes
            .orphan_node(DomNode::Element(Element::new(name, attrs)))
            .id;

        if is_fragment {
            self.nodes.append_child(node_id, DomNode::Fragment);
        }

        node_id
    }

    fn create_comment(&mut self, text: html5ever::tendril::StrTendril) -> Self::Handle {
        self.nodes
            .orphan_node(DomNode::Comment(Comment::new(text)))
            .id
    }

    fn create_pi(
        &mut self,
        target: html5ever::tendril::StrTendril,
        data: html5ever::tendril::StrTendril,
    ) -> Self::Handle {
        self.nodes
            .orphan_node(DomNode::ProcessingInstruction(ProcessingInstruction::new(
                target, data,
            )))
            .id
    }

    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match child {
            NodeOrText::AppendNode(n) => {
                self.nodes.append_child_id(*parent, n).unwrap();
            }
            NodeOrText::AppendText(txt) => {
                if let Some((_, last)) = self.nodes.children_range(*parent) {
                    if let DomNode::Text(last_node) =
                        &mut self.nodes.node_mut_ref(last).unwrap().data
                    {
                        last_node.push_tendril(&txt);
                        return;
                    }
                }

                self.nodes
                    .append_child(*parent, DomNode::Text(Text::new(txt)));
            }
        };
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        match self.nodes.parent_ref(*element) {
            Some(_) => self.append_before_sibling(element, child),
            None => self.append(prev_element, child),
        }
    }

    fn append_doctype_to_document(
        &mut self,
        name: html5ever::tendril::StrTendril,
        public_id: html5ever::tendril::StrTendril,
        system_id: html5ever::tendril::StrTendril,
    ) {
        self.nodes.append_child(
            self.nodes.root_ref().unwrap().id,
            DomNode::DocType(DocType::new(name, public_id, system_id)),
        );
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        self.nodes.children_range(*target).unwrap().0
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&mut self, mode: html5ever::tree_builder::QuirksMode) {
        self.quirks_mode = mode
    }

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: NodeOrText<Self::Handle>,
    ) {
        if let NodeOrText::AppendNode(new_node_id) = new_node {
            self.nodes.detach(new_node_id);
        }

        // let have_parent =
        if self.nodes.parent_ref(*sibling).is_some() {
            match new_node {
                NodeOrText::AppendNode(new_node_id) => {
                    self.nodes.insert_id_before(*sibling, new_node_id);
                }
                NodeOrText::AppendText(txt) => {
                    if let Some(old_prev_sib) = self.nodes.previous_sibling_ref(*sibling) {
                        if let DomNode::Text(t) =
                            &mut self.nodes.node_mut_ref(old_prev_sib.id).unwrap().data
                        {
                            t.push_tendril(&txt);
                            return;
                        }
                    }

                    self.nodes
                        .insert_before(*sibling, DomNode::Text(Text::new(txt)));
                }
            }
        }
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<html5ever::Attribute>) {
        let node = self.nodes.node_mut_ref(*target).unwrap();
        match &mut node.data {
            DomNode::Element(e) => e.add_attrs(attrs),
            _ => unreachable!(),
        }
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        self.nodes.detach(*target).unwrap();
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        self.nodes
            .reparent_from_id_append(*node, *new_parent)
            .unwrap();
    }
}
