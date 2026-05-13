use crate::engine::{Element, Key};
use crate::render_contract::ElementKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Cursor {
    UnderlineDots { count: usize, focus: usize },
    RowGlow { index: usize },
}

impl Cursor {
    pub fn element(&self) -> Element {
        match self {
            Self::UnderlineDots { count, focus } => {
                let selected = (*focus).min(count.saturating_sub(1));
                (0..*count).fold(
                    Element::new(ElementKind::Container, Some("cursor_dots"))
                        .key(Key::Static("cursor")),
                    |element, index| {
                        element.child(
                            Element::new(ElementKind::Container, Some("cursor_dot"))
                                .key(Key::Indexed(index))
                                .selected(index == selected),
                        )
                    },
                )
            }
            Self::RowGlow { index } => {
                Element::new(ElementKind::Container, Some("cursor_row_glow"))
                    .key(Key::Static("cursor"))
                    .offset_y((*index as i32) * 36)
            }
        }
    }
}
