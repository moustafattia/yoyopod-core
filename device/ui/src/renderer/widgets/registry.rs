use std::collections::HashMap;
use std::ptr::NonNull;

use anyhow::{anyhow, Result};

use crate::renderer::lvgl::ffi;
use crate::renderer::widgets::WidgetId;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Layout {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WidgetKind {
    Root,
    Container,
    Label,
    Image,
}

#[derive(Debug)]
pub(crate) struct WidgetNode {
    pub obj: NonNull<ffi::lv_obj_t>,
    pub kind: WidgetKind,
    pub role: &'static str,
    pub children: Vec<WidgetId>,
    pub layout: Layout,
    pub y_offset: i32,
}

#[derive(Debug, Default)]
pub(crate) struct WidgetRegistry {
    next_widget_id: u64,
    widgets: HashMap<WidgetId, WidgetNode>,
}

impl WidgetRegistry {
    pub fn obj(&self, widget: WidgetId) -> Result<NonNull<ffi::lv_obj_t>> {
        self.widgets
            .get(&widget)
            .map(|node| node.obj)
            .ok_or_else(|| anyhow!("unknown LVGL widget {}", widget.raw()))
    }

    pub fn node_mut(&mut self, widget: WidgetId) -> Result<&mut WidgetNode> {
        self.widgets
            .get_mut(&widget)
            .ok_or_else(|| anyhow!("unknown LVGL widget {}", widget.raw()))
    }

    pub fn register(
        &mut self,
        obj: NonNull<ffi::lv_obj_t>,
        kind: WidgetKind,
        role: &'static str,
        parent: Option<WidgetId>,
        layout: Layout,
    ) -> WidgetId {
        let id = self.next_widget_id();
        self.widgets.insert(
            id,
            WidgetNode {
                obj,
                kind,
                role,
                children: Vec::new(),
                layout,
                y_offset: 0,
            },
        );
        if let Some(parent) = parent {
            if let Some(parent_node) = self.widgets.get_mut(&parent) {
                parent_node.children.push(id);
            }
        }
        id
    }

    pub fn remove_subtree(&mut self, widget: WidgetId) {
        if let Some(node) = self.widgets.remove(&widget) {
            for child in node.children {
                self.remove_subtree(child);
            }
        }
    }

    pub fn clear(&mut self) {
        self.widgets.clear();
    }

    fn next_widget_id(&mut self) -> WidgetId {
        let id = WidgetId::new(self.next_widget_id);
        self.next_widget_id += 1;
        id
    }
}
