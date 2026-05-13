use crate::engine::Element;
use crate::render_contract::ElementKind;

pub fn image(role: &'static str) -> Element {
    Element::new(ElementKind::Image, Some(role))
}
