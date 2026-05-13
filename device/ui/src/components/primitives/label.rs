use crate::engine::{Element, ElementKind};

pub fn label(role: &'static str) -> Element {
    Element::new(ElementKind::Label, Some(role))
}
