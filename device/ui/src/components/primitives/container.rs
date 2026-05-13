use crate::engine::{Element, ElementKind};

pub fn container(role: &'static str) -> Element {
    Element::new(ElementKind::Container, Some(role))
}
