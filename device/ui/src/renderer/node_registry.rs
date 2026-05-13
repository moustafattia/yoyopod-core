use std::collections::BTreeMap;

use crate::engine::NodeId;
use crate::renderer::widgets::WidgetId;

#[derive(Debug, Default)]
pub struct NodeRegistry {
    widgets: BTreeMap<NodeId, WidgetId>,
}

impl NodeRegistry {
    pub fn widget(&self, node: NodeId) -> Option<WidgetId> {
        self.widgets.get(&node).copied()
    }

    pub fn bind(&mut self, node: NodeId, widget: WidgetId) {
        self.widgets.insert(node, widget);
    }

    pub fn remove(&mut self, node: NodeId) -> Option<WidgetId> {
        self.widgets.remove(&node)
    }

    pub fn clear(&mut self) {
        self.widgets.clear();
    }

    pub fn len(&self) -> usize {
        self.widgets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.widgets.is_empty()
    }
}
