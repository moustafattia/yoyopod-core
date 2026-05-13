use crate::engine::Element;
use crate::render_contract::ElementKind;

pub fn label(role: &'static str) -> Element {
    Element::new(ElementKind::Label, Some(role))
}
