use super::{ElementKind, NodeId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mutation {
    Create {
        node: NodeId,
        parent: NodeId,
        kind: ElementKind,
        role: Option<&'static str>,
    },
    Update {
        node: NodeId,
        prop: PropChange,
    },
    Place {
        node: NodeId,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    },
    Reorder {
        parent: NodeId,
        order: Vec<NodeId>,
    },
    Remove {
        node: NodeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropChange {
    Text(String),
    Icon(String),
    Accent(u32),
    Selected(bool),
    Visible(bool),
    Opacity(u8),
    OffsetX(i32),
    OffsetY(i32),
    ScalePermille(i32),
    SelectionOffset(i32),
    Variant(&'static str),
    Progress(i32),
}
