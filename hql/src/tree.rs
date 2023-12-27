use std::fmt::{Debug, Display};

use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeID(usize);

impl From<usize> for NodeID {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<NodeID> for usize {
    fn from(val: NodeID) -> Self {
        val.0
    }
}

impl Display for NodeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Node is the the virtual DOM-node partially
/// following https://dom.spec.whatwg.org/#interface-node standard.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Node<T: Debug + Display> {
    pub id: NodeID,
    pub data: T,

    parent: Option<NodeID>,
    children: Option<(NodeID, NodeID)>, // nodeID of the first and the last child
    previous_sibling: Option<NodeID>,
    next_sibling: Option<NodeID>,
}

impl<T: Debug + Display> Display for Node<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl<T: Debug + Display> Node<T> {
    pub fn orphan<I: Into<NodeID>>(id: I, data: T) -> Self {
        Node {
            id: id.into(),
            data,
            parent: None,
            children: None,
            previous_sibling: None,
            next_sibling: None,
        }
    }

    pub fn phantom(data: T) -> Self {
        Self::orphan(usize::MAX, data)
    }
}

#[derive(Debug)]
pub struct Tree<T: Debug + Display> {
    nodes: Vec<Node<T>>,
}

impl<T: Debug + Display> Tree<T> {
    pub fn new(root: T) -> Self {
        Tree {
            nodes: vec![Node::orphan(0, root)],
        }
    }

    pub fn nodes(&self) -> &Vec<Node<T>> {
        &self.nodes
    }

    pub fn orphan_node(&mut self, data: T) -> &Node<T> {
        let node_id = self.nodes.len();
        self.nodes.push(Node::orphan(node_id, data));
        self.node_ref(node_id.into()).unwrap()
    }

    /// Just wrap the data into node. It will not store it in the tree and build any connections
    /// to other nodes. The NodeID is usize::MAX as notation.
    pub fn phantom_node(&self, data: T) -> Node<T> {
        Node::orphan(usize::MAX, data)
    }

    pub fn node_ref(&self, id: NodeID) -> Option<&Node<T>> {
        self.nodes.get::<usize>(id.into())
    }

    pub fn node_mut_ref(&mut self, id: NodeID) -> Option<&mut Node<T>> {
        self.nodes.get_mut::<usize>(id.into())
    }

    pub fn root_ref(&self) -> Option<&Node<T>> {
        self.node_ref(0.into())
    }

    pub fn previous_sibling_ref(&self, node_id: NodeID) -> Option<&Node<T>> {
        self.node_ref(self.node_ref(node_id)?.previous_sibling?)
    }

    pub fn parent_ref(&self, id: NodeID) -> Option<&Node<T>> {
        let parent = self.node_ref(id)?.parent?;
        self.node_ref(parent)
    }

    pub fn children_range(&self, parent: NodeID) -> Option<(NodeID, NodeID)> {
        self.node_ref(parent)?.children
    }

    /// Insert new_sib_id as new previous sibling of node_id
    ///
    /// Return reference of the new sibling
    pub fn insert_id_before(&mut self, node_id: NodeID, new_sib_id: NodeID) -> Option<&Node<T>> {
        let parent_id = self.parent_ref(node_id)?.id;
        let old_sib = self.node_ref(node_id)?.previous_sibling;

        let new_sib = self.node_mut_ref(new_sib_id).unwrap();
        new_sib.previous_sibling = old_sib;
        new_sib.next_sibling = Some(node_id);
        new_sib.parent = Some(parent_id);

        if let Some(old_sib_id) = old_sib {
            // change next sibling pointer of old sibling to the new sibling
            self.node_mut_ref(old_sib_id).unwrap().next_sibling = Some(new_sib_id)
        } else {
            // update parent
            let parent = self.node_mut_ref(parent_id).unwrap();

            parent.children = Some((new_sib_id, parent.children.unwrap().1));
        }

        let node = self.node_mut_ref(node_id).unwrap();

        // update the prev_sibling of current node, pointing to new_sib
        node.previous_sibling = Some(new_sib_id);

        self.node_ref(new_sib_id)
    }

    /// Inserts a sibling before node_id
    ///
    /// Return None if node_id or its parent does not exist
    pub fn insert_before(&mut self, node_id: NodeID, data: T) -> Option<&Node<T>> {
        let new_sib_id = self.orphan_node(data).id;
        self.insert_id_before(node_id, new_sib_id)
    }

    /// Append child as the last child to the target. It will first detach the old child.
    ///
    /// Return reference of `child`
    ///
    /// Return None if child or target do not exist
    pub fn append_child_id(&mut self, target: NodeID, child: NodeID) -> Option<&Node<T>> {
        // check whether target and child exist
        self.node_ref(child)?;
        self.node_mut_ref(target)?;

        // clear old info
        self.detach(child);

        match self.children_range(target) {
            Some((first, last)) => {
                // refresh child info
                let child_node = self.node_mut_ref(child).unwrap();
                child_node.parent = Some(target);
                child_node.previous_sibling = Some(last);
                child_node.next_sibling = None;

                // update last child, pointing to child
                let last_child = self.node_mut_ref(last).unwrap();
                last_child.next_sibling = Some(child);

                // update last child nodeID for parent
                let parent = self.node_mut_ref(target).unwrap();
                parent.children = Some((first, child));
            }
            None => {
                {}
                // refresh child sibling info
                let child_node = self.node_mut_ref(child).unwrap();
                child_node.parent = Some(target);
                child_node.previous_sibling = None;
                child_node.next_sibling = None;

                // since child is the only children of target. init parent
                let parent = self.node_mut_ref(target).unwrap();
                parent.children = Some((child, child));
            }
        };

        self.node_ref(child)
    }

    /// Appends `child` as the last child of target
    ///
    /// Return reference of the child
    ///
    /// Return None if target does not exist
    pub fn append_child(&mut self, target: NodeID, child: T) -> Option<&Node<T>> {
        let id = self.orphan_node(child).id;
        self.append_child_id(target, id)
    }

    /// Remove all the children from node and append them to new_parent.
    pub fn reparent_from_id_append(
        &mut self,
        node: NodeID,
        new_parent: NodeID,
    ) -> Option<&Node<T>> {
        // check whether node and new_parent exist
        self.node_ref(node)?;
        self.node_ref(new_parent)?;

        // if old node does not have children, then just ignore
        if let Some((first, last)) = self.children_range(node) {
            // assign parents of old children to new one
            let mut child = Some(first);
            // do-while black magic
            while {
                let child_node = self.node_mut_ref(child.unwrap()).unwrap();
                child_node.parent = Some(new_parent);
                child = child_node.next_sibling;

                // child.is_some_and(|c| c != last)
                child.is_some()
            } {}

            // change children nodeID range of new parent
            match self.children_range(new_parent) {
                Some((new_first, new_last)) => {
                    let parent = self.node_mut_ref(new_parent).unwrap();
                    parent.children = Some((new_first, last));

                    let old_first_node = self.node_mut_ref(first).unwrap();
                    old_first_node.previous_sibling = Some(new_last);

                    let new_last_node = self.node_mut_ref(new_last).unwrap();
                    new_last_node.next_sibling = Some(first);
                }
                None => {
                    let parent = self.node_mut_ref(new_parent).unwrap();
                    parent.children = Some((first, last));
                }
            };
        }

        self.node_ref(new_parent)
    }

    // Detach this node from its parent
    pub fn detach(&mut self, node_id: NodeID) -> Option<&Node<T>> {
        self.node_ref(node_id)?;

        let parent_id = self.parent_ref(node_id).and_then(|n| n.parent);

        // only handle case that parent exists
        if let Some(parent_id) = parent_id {
            let node = self.node_mut_ref(node_id).unwrap();
            let prev_sibling_id = node.previous_sibling;
            let next_sibling_id = node.next_sibling;

            // clean node relations
            node.parent = None;
            node.previous_sibling = None;
            node.next_sibling = None;

            // connect prev_sibling and next_sib
            if let Some(prev_sib_id) = prev_sibling_id {
                let prev_sib = self.node_mut_ref(prev_sib_id).unwrap();
                prev_sib.next_sibling = next_sibling_id;
            }
            if let Some(next_sib_id) = next_sibling_id {
                let next_sib = self.node_mut_ref(next_sib_id).unwrap();
                next_sib.previous_sibling = prev_sibling_id;
            }

            // change children fields of parent
            let parent = self.node_mut_ref(parent_id).unwrap();
            let (first, last) = parent.children.unwrap();

            if first == last {
                parent.children = None;
            } else if first == node_id {
                parent.children = Some((next_sibling_id.unwrap(), last));
            } else if last == node_id {
                parent.children = Some((first, prev_sibling_id.unwrap()));
            }
        }

        self.node_ref(node_id)
    }
}

pub struct ChildrenTraverse<'a, T: Debug + Display> {
    tree: &'a Tree<T>,

    cur: Option<&'a Node<T>>,

    reversed: bool,
}

impl<'a, T: Debug + Display> ChildrenTraverse<'a, T> {
    pub fn new(tree: &'a Tree<T>, parent: &'a Node<T>, reversed: bool) -> Self {
        Self {
            tree,
            cur: parent.children.and_then(|(first, last)| {
                tree.node_ref(match reversed {
                    false => first,
                    true => last,
                })
            }),
            reversed,
        }
    }
}

impl<'a, T: Debug + Display> Iterator for ChildrenTraverse<'a, T> {
    type Item = (&'a Node<T>, &'a Tree<T>);

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur;

        if let Some(cur) = cur {
            self.cur = match self.reversed {
                true => cur.previous_sibling.and_then(|id| self.tree.node_ref(id)),
                false => cur.next_sibling.and_then(|id| self.tree.node_ref(id)),
            }
        }

        cur.map(|c| (c, self.tree))
    }
}

pub struct PreOrderTraverse<'a, T: Debug + Display> {
    tree: &'a Tree<T>,

    root: &'a Node<T>,
    cur: Option<&'a Node<T>>,
}

impl<'a, T: Debug + Display> PreOrderTraverse<'a, T> {
    pub fn new(tree: &'a Tree<T>, root: &'a Node<T>) -> Self {
        Self {
            tree,
            root,
            cur: Some(root),
        }
    }
}

impl<'a, T: Debug + Display> Iterator for PreOrderTraverse<'a, T> {
    type Item = (&'a Node<T>, &'a Tree<T>);

    fn next(&mut self) -> Option<Self::Item> {
        let cur = self.cur?;
        info!("visit: {:?}", cur);

        match cur.children {
            Some((first, _)) => {
                info!("{} has children. go to next depth", first);
                self.cur = self.tree.node_ref(first);
            }

            None => {
                match cur.next_sibling.and_then(|n| self.tree.node_ref(n)) {
                    Some(sib) => {
                        info!("{} no child. to sibling", cur.id);
                        self.cur = Some(sib);
                    }
                    // TODO(xylonx): refactor for readability
                    None => {
                        info!("{} no child and sibling. to parent's sibling", cur.id);
                        let mut parent = cur.parent.and_then(|n| self.tree.node_ref(n));
                        loop {
                            match parent {
                                Some(p) => {
                                    if p.id == self.root.id {
                                        self.cur = None;
                                        break;
                                    }
                                    match p.next_sibling {
                                        Some(sib) => {
                                            self.cur = self.tree.node_ref(sib);
                                            break;
                                        }
                                        None => {
                                            parent = p.parent.and_then(|n| self.tree.node_ref(n))
                                        }
                                    };
                                }
                                None => {
                                    self.cur = None;
                                    break;
                                }
                            }
                        }
                    }
                };
            }
        }

        Some((cur, self.tree))
    }
}

#[cfg(test)]
mod test {
    use crate::tree::ChildrenTraverse;

    use super::{PreOrderTraverse, Tree};

    #[test]
    fn test_tree_preorder_traverse() {
        let mut tree = Tree::new(0);
        let root = tree.root_ref().unwrap().id;

        let node1 = tree.append_child(root, 1).unwrap().id;
        tree.append_child(root, 2).unwrap();
        let node3 = tree.append_child(root, 3).unwrap().id;

        let node4 = tree.append_child(node1, 4).unwrap().id;
        let node5 = tree.append_child(node4, 5).unwrap().id;
        tree.append_child(node5, 6).unwrap();

        let node7 = tree.append_child(node3, 7).unwrap().id;
        tree.append_child(node7, 8).unwrap();
        tree.append_child(node7, 9).unwrap();

        let node_values = PreOrderTraverse::new(&tree, tree.root_ref().unwrap())
            .map(|(n, _)| n.data)
            .collect::<Vec<_>>();
        let preorder_values = vec![0, 1, 4, 5, 6, 2, 3, 7, 8, 9];
        assert_eq!(
            node_values, preorder_values,
            "want: {:?}, get: {:?}",
            preorder_values, node_values,
        )
    }

    #[test]
    fn test_tree_children_traverse() {
        let mut tree = Tree::new(0);
        let root = tree.root_ref().unwrap().id;
        let node1 = tree.append_child(root, 1).unwrap().id;
        let node2 = tree.append_child(root, 2).unwrap().id;

        tree.append_child(node1, 3);
        tree.append_child(node2, 4);

        let node_ids = ChildrenTraverse::new(&tree, tree.root_ref().unwrap(), false)
            .map(|(n, _)| n.id.0)
            .collect::<Vec<_>>();
        let preorder_ids = vec![1, 2];
        assert_eq!(
            node_ids, preorder_ids,
            "want: {:?}, get: {:?}",
            preorder_ids, node_ids,
        )
    }
}
