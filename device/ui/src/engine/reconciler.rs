use crate::animation::TimelineSampler;

use super::{props, Element, ElementProps, Layout, Mutation, NodeId, PropChange};

#[derive(Debug, Default)]
pub struct Reconciler;

impl Reconciler {
    pub fn diff(
        &mut self,
        previous: Option<&Element>,
        next: &Element,
        _sampler: &TimelineSampler<'_>,
        out: &mut Vec<Mutation>,
    ) {
        out.clear();
        let mut next_id = 0;
        diff_element(previous, next, NodeId(0), &mut next_id, out);
    }
}

fn diff_element(
    previous: Option<&Element>,
    next: &Element,
    parent: NodeId,
    next_id: &mut u32,
    out: &mut Vec<Mutation>,
) -> NodeId {
    let node = alloc(next_id);
    match previous {
        None => create_subtree(node, parent, next, next_id, out),
        Some(previous) if should_replace(previous, next) => {
            out.push(Mutation::Remove { node });
            create_subtree(node, parent, next, next_id, out);
        }
        Some(previous) => {
            emit_prop_updates(node, &previous.props, &next.props, out);
            emit_place(node, next.layout, out);
            diff_children(previous, next, node, next_id, out);
        }
    }
    node
}

fn create_subtree(
    node: NodeId,
    parent: NodeId,
    element: &Element,
    next_id: &mut u32,
    out: &mut Vec<Mutation>,
) {
    out.push(Mutation::Create {
        node,
        parent,
        kind: element.kind,
        role: element.role,
    });
    emit_prop_updates(node, &ElementProps::default(), &element.props, out);
    emit_place(node, element.layout, out);
    for child in &element.children {
        let child_node = alloc(next_id);
        create_subtree(child_node, node, child, next_id, out);
    }
}

fn diff_children(
    previous: &Element,
    next: &Element,
    parent: NodeId,
    next_id: &mut u32,
    out: &mut Vec<Mutation>,
) {
    let mut order = Vec::with_capacity(next.children.len());
    for (index, child) in next.children.iter().enumerate() {
        let previous_child = previous.children.get(index);
        let node = diff_element(previous_child, child, parent, next_id, out);
        order.push(node);
    }
    for _ in next.children.len()..previous.children.len() {
        out.push(Mutation::Remove {
            node: alloc(next_id),
        });
    }
    if !order.is_empty() {
        out.push(Mutation::Reorder { parent, order });
    }
}

fn emit_prop_updates(
    node: NodeId,
    previous: &ElementProps,
    next: &ElementProps,
    out: &mut Vec<Mutation>,
) {
    let mut changes = Vec::new();
    props::diff_props(previous, next, &mut changes);
    for prop in changes {
        out.push(Mutation::Update { node, prop });
    }
    if next.text.is_none() && previous.text.is_some() {
        out.push(Mutation::Update {
            node,
            prop: PropChange::Text(String::new()),
        });
    }
    if next.icon_key.is_none() && previous.icon_key.is_some() {
        out.push(Mutation::Update {
            node,
            prop: PropChange::Icon(String::new()),
        });
    }
}

fn emit_place(node: NodeId, layout: Layout, out: &mut Vec<Mutation>) {
    if let Layout::Absolute { x, y, w, h } = layout {
        out.push(Mutation::Place { node, x, y, w, h });
    }
}

fn should_replace(previous: &Element, next: &Element) -> bool {
    previous.key != next.key || previous.kind != next.kind || previous.role != next.role
}

fn alloc(next_id: &mut u32) -> NodeId {
    let node = NodeId(*next_id);
    *next_id = next_id.saturating_add(1);
    node
}
