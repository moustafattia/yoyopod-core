use super::{Element, NodeId};

#[derive(Debug, Default)]
pub struct TreeCache {
    previous: Option<Element>,
}

impl TreeCache {
    pub fn previous(&self) -> Option<&Element> {
        self.previous.as_ref()
    }

    pub fn replace(&mut self, tree: Element) {
        self.previous = Some(tree);
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
