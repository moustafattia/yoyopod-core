use crate::animation::{AnimatableProp, AnimatableValue, TimelineSampler};
use crate::render_contract::{Mutation, NodeId, PropChange};
use crate::scene::{region_rect, RegionId, Stage};

use std::collections::{BTreeMap, BTreeSet};

use super::{props, Element, ElementProps, Layout, NodeIdAlloc, NodePath};

#[derive(Debug, Default)]
pub struct Reconciler;

impl Reconciler {
    pub fn diff(
        &mut self,
        previous: Option<&Element>,
        previous_ids: &BTreeMap<NodePath, NodeId>,
        next: &Element,
        stage: Stage,
        node_alloc: &mut NodeIdAlloc,
        sampler: &TimelineSampler<'_>,
        out: &mut Vec<Mutation>,
    ) -> BTreeMap<NodePath, NodeId> {
        out.clear();
        let mut next_ids = BTreeMap::new();
        let root_path = NodePath::root(next);
        diff_element(
            previous,
            next,
            NodeId(0),
            root_path,
            previous_ids,
            &mut next_ids,
            node_alloc,
            stage,
            sampler,
            out,
        );
        next_ids
    }
}

fn diff_element(
    previous: Option<&Element>,
    next: &Element,
    parent: NodeId,
    path: NodePath,
    previous_ids: &BTreeMap<NodePath, NodeId>,
    next_ids: &mut BTreeMap<NodePath, NodeId>,
    node_alloc: &mut NodeIdAlloc,
    stage: Stage,
    sampler: &TimelineSampler<'_>,
    out: &mut Vec<Mutation>,
) -> NodeId {
    let previous_node = previous_ids.get(&path).copied();
    let replace = previous
        .map(|previous| should_replace(previous, next))
        .unwrap_or(false);
    let node = if replace {
        node_alloc.alloc()
    } else {
        previous_node.unwrap_or_else(|| node_alloc.alloc())
    };
    next_ids.insert(path.clone(), node);

    match previous {
        None => create_subtree(
            node, parent, next, path, next_ids, node_alloc, stage, sampler, out,
        ),
        Some(previous) if replace => {
            if let Some(previous_node) = previous_node {
                remove_subtree(previous, path.clone(), previous_node, previous_ids, out);
            }
            create_subtree(
                node, parent, next, path, next_ids, node_alloc, stage, sampler, out,
            );
        }
        Some(previous) => {
            emit_prop_updates(node, &previous.props, &next.props, out);
            emit_place(node, next.layout, stage, out);
            emit_animation_updates(node, next, sampler, out);
            diff_children(
                previous,
                next,
                node,
                path,
                previous_ids,
                next_ids,
                node_alloc,
                stage,
                sampler,
                out,
            );
        }
    }
    node
}

fn create_subtree(
    node: NodeId,
    parent: NodeId,
    element: &Element,
    path: NodePath,
    next_ids: &mut BTreeMap<NodePath, NodeId>,
    node_alloc: &mut NodeIdAlloc,
    stage: Stage,
    sampler: &TimelineSampler<'_>,
    out: &mut Vec<Mutation>,
) {
    next_ids.insert(path.clone(), node);
    out.push(Mutation::Create {
        node,
        parent,
        kind: element.kind,
        role: element.role,
    });
    emit_prop_updates(node, &ElementProps::default(), &element.props, out);
    emit_place(node, element.layout, stage, out);
    emit_animation_updates(node, element, sampler, out);
    for (index, child) in element.children.iter().enumerate() {
        let child_path = path.child(child, index);
        let child_node = node_alloc.alloc();
        create_subtree(
            child_node, node, child, child_path, next_ids, node_alloc, stage, sampler, out,
        );
    }
}

fn diff_children(
    previous: &Element,
    next: &Element,
    parent: NodeId,
    parent_path: NodePath,
    previous_ids: &BTreeMap<NodePath, NodeId>,
    next_ids: &mut BTreeMap<NodePath, NodeId>,
    node_alloc: &mut NodeIdAlloc,
    stage: Stage,
    sampler: &TimelineSampler<'_>,
    out: &mut Vec<Mutation>,
) {
    let mut order = Vec::with_capacity(next.children.len());
    let mut matched_previous = BTreeSet::new();
    for (index, child) in next.children.iter().enumerate() {
        let child_path = parent_path.child(child, index);
        let previous_match =
            previous
                .children
                .iter()
                .enumerate()
                .find(|(previous_index, previous_child)| {
                    parent_path.child(previous_child, *previous_index) == child_path
                });
        let previous_child = previous_match.map(|(previous_index, previous_child)| {
            matched_previous.insert(previous_index);
            previous_child
        });
        let node = diff_element(
            previous_child,
            child,
            parent,
            child_path,
            previous_ids,
            next_ids,
            node_alloc,
            stage,
            sampler,
            out,
        );
        order.push(node);
    }
    for (index, child) in previous.children.iter().enumerate() {
        if matched_previous.contains(&index) {
            continue;
        }
        let child_path = parent_path.child(child, index);
        if let Some(node) = previous_ids.get(&child_path).copied() {
            remove_subtree(child, child_path, node, previous_ids, out);
        }
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

fn emit_place(node: NodeId, layout: Layout, stage: Stage, out: &mut Vec<Mutation>) {
    let Some((x, y, w, h)) = resolve_layout(stage, layout) else {
        return;
    };
    out.push(Mutation::Place { node, x, y, w, h });
}

fn resolve_layout(stage: Stage, layout: Layout) -> Option<(i32, i32, i32, i32)> {
    match layout {
        Layout::Absolute { x, y, w, h } => Some((x, y, w, h)),
        Layout::Region(RegionId::Auto) => None,
        Layout::Region(region) => {
            let rect = region_rect(stage, region)
                .unwrap_or_else(|| panic!("stage {stage:?} has no rect for region {region:?}"));
            Some((rect.x, rect.y, rect.w, rect.h))
        }
    }
}

fn emit_animation_updates(
    node: NodeId,
    element: &Element,
    sampler: &TimelineSampler<'_>,
    out: &mut Vec<Mutation>,
) {
    let Some(anim) = element.anim else {
        return;
    };
    let Some((property, value)) = sampler.slot_value(anim.timeline, anim.track) else {
        return;
    };
    if let Some(prop) = prop_change_for_animation(property, value) {
        out.push(Mutation::Update { node, prop });
    }
}

fn prop_change_for_animation(
    property: AnimatableProp,
    value: AnimatableValue,
) -> Option<PropChange> {
    match (property, value) {
        (AnimatableProp::Opacity, AnimatableValue::U8(value)) => Some(PropChange::Opacity(value)),
        (AnimatableProp::ProgressPermille, AnimatableValue::I32(value)) => {
            Some(PropChange::Progress(value))
        }
        (AnimatableProp::AccentMix, AnimatableValue::Rgb(value)) => Some(PropChange::Accent(value)),
        _ => None,
    }
}

fn remove_subtree(
    element: &Element,
    path: NodePath,
    node: NodeId,
    previous_ids: &BTreeMap<NodePath, NodeId>,
    out: &mut Vec<Mutation>,
) {
    for (index, child) in element.children.iter().enumerate() {
        let child_path = path.child(child, index);
        if let Some(child_node) = previous_ids.get(&child_path).copied() {
            remove_subtree(child, child_path, child_node, previous_ids, out);
        }
    }
    out.push(Mutation::Remove { node });
}

fn should_replace(previous: &Element, next: &Element) -> bool {
    previous.key != next.key || previous.kind != next.kind || previous.role != next.role
}
