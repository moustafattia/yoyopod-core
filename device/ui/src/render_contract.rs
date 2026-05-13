#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRegion {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl DirtyRegion {
    pub const fn union(self, other: Self) -> Self {
        let x0 = if self.x < other.x { self.x } else { other.x };
        let y0 = if self.y < other.y { self.y } else { other.y };
        let self_x1 = self.x.saturating_add(self.w);
        let self_y1 = self.y.saturating_add(self.h);
        let other_x1 = other.x.saturating_add(other.w);
        let other_y1 = other.y.saturating_add(other.h);
        let x1 = if self_x1 > other_x1 {
            self_x1
        } else {
            other_x1
        };
        let y1 = if self_y1 > other_y1 {
            self_y1
        } else {
            other_y1
        };
        Self {
            x: x0,
            y: y0,
            w: x1.saturating_sub(x0),
            h: y1.saturating_sub(y0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ElementKind {
    Container,
    Label,
    Image,
    Progress,
}

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
    Variant(&'static str),
    Progress(i32),
}
