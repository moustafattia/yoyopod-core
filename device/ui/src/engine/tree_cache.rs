use std::collections::BTreeMap;

use crate::render_contract::{ElementKind, NodeId};

use super::{Element, Key};

#[derive(Debug, Default)]
pub struct TreeCache {
    previous: Option<Element>,
    ids: BTreeMap<NodePath, NodeId>,
}

impl TreeCache {
    pub fn previous(&self) -> Option<&Element> {
        self.previous.as_ref()
    }

    pub fn ids(&self) -> &BTreeMap<NodePath, NodeId> {
        &self.ids
    }

    pub fn replace(&mut self, tree: Element, ids: BTreeMap<NodePath, NodeId>) {
        self.previous = Some(tree);
        self.ids = ids;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct NodePath(Vec<NodePathSegment>);

impl NodePath {
    pub fn root(element: &Element) -> Self {
        Self(vec![NodePathSegment::from_element(element, 0)])
    }

    pub fn child(&self, element: &Element, sibling_index: usize) -> Self {
        let mut path = self.0.clone();
        path.push(NodePathSegment::from_element(element, sibling_index));
        Self(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum NodePathSegment {
    Key(Key),
    Position {
        kind: ElementKind,
        role: Option<&'static str>,
        sibling_index: usize,
    },
}

impl NodePathSegment {
    fn from_element(element: &Element, sibling_index: usize) -> Self {
        if let Some(key) = &element.key {
            return Self::Key(key.clone());
        }
        Self::Position {
            kind: element.kind,
            role: element.role,
            sibling_index,
        }
    }
}

#[derive(Debug, Default)]
pub struct NodeIdAlloc {
    next: u32,
}

impl NodeIdAlloc {
    pub fn alloc(&mut self) -> NodeId {
        let id = NodeId(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}
