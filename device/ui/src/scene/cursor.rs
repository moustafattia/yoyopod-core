use crate::engine::{Element, ElementKind, Key};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cursor {
    UnderlineDots { count: usize, focus: usize },
    RowGlow,
}

impl Cursor {
    pub fn element(&self) -> Element {
        match self {
            Self::UnderlineDots { count, focus } => {
                Element::new(ElementKind::Container, Some("cursor_dots"))
                    .key(Key::Static("cursor"))
                    .text(format!("{focus}/{count}"))
            }
            Self::RowGlow => Element::new(ElementKind::Container, Some("cursor_row_glow"))
                .key(Key::Static("cursor")),
        }
    }
}
