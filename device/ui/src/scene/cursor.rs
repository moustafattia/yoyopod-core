use crate::animation::ActorRef;
use crate::engine::{Element, Key};
use crate::render_contract::ElementKind;
use crate::scene::roles;

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
                    Element::new(ElementKind::Container, Some(roles::CURSOR_DOTS))
                        .key(Key::Static("cursor"))
                        .actor(ActorRef::Cursor),
                    |element, index| {
                        element.child(
                            Element::new(ElementKind::Container, Some(roles::CURSOR_DOT))
                                .key(Key::Indexed(index))
                                .selected(index == selected),
                        )
                    },
                )
            }
            Self::RowGlow { index } => {
                Element::new(ElementKind::Container, Some(roles::CURSOR_ROW_GLOW))
                    .key(Key::Static("cursor"))
                    .actor(ActorRef::Cursor)
                    .offset_y((*index as i32) * 36)
            }
        }
    }
}
