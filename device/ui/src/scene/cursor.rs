use crate::engine::{Element, Key};
use crate::render_contract::ElementKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cursor {
    UnderlineDots { count: usize, focus: usize },
    RowGlow,
}

impl Cursor {
    pub fn element(&self) -> Element {
        match self {
            Self::UnderlineDots { .. } => {
                Element::new(ElementKind::Container, Some("cursor_dots")).key(Key::Static("cursor"))
            }
            Self::RowGlow => Element::new(ElementKind::Container, Some("cursor_row_glow"))
                .key(Key::Static("cursor")),
        }
    }
}
