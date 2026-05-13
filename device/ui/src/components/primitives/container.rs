use crate::engine::Element;
use crate::render_contract::ElementKind;

pub fn container(role: &'static str) -> Element {
    Element::new(ElementKind::Container, Some(role))
}
