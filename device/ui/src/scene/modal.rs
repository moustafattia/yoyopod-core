use crate::engine::{Element, Key};
use crate::render_contract::ElementKind;

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
    let mut element = Element::new(ElementKind::Container, Some("modal")).key(Key::Indexed(index));
    element.props.variant = Some(variant);
    element
        .child(Element::new(ElementKind::Label, Some("modal_title")).text(title))
        .child(Element::new(ElementKind::Label, Some("modal_message")).text(message))
}
