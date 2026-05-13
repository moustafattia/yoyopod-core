use crate::engine::{Element, ElementKind};

pub fn image(role: &'static str) -> Element {
    Element::new(ElementKind::Image, Some(role))
}
