use crate::engine::{Element, Key};
use crate::scene::roles;
use crate::ElementKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modal {
    Loading { title: String, message: String },
    Error { title: String, message: String },
}

impl Modal {
    pub fn element(&self, index: usize) -> Element {
        match self {
            Self::Loading { title, message } => modal_content(index, "loading", title, message),
            Self::Error { title, message } => modal_content(index, "error", title, message),
        }
    }
}

fn modal_content(index: usize, variant: &'static str, title: &str, message: &str) -> Element {
    let mut element =
        Element::new(ElementKind::Container, Some(roles::MODAL)).key(Key::Indexed(index));
    element.props.variant = Some(variant);
    element
        .child(Element::new(ElementKind::Label, Some(roles::MODAL_TITLE)).text(title))
        .child(Element::new(ElementKind::Label, Some(roles::MODAL_MESSAGE)).text(message))
}
