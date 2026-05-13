use crate::engine::{Element, ElementKind};

pub fn progress(role: &'static str, value: i32) -> Element {
    let mut element = Element::new(ElementKind::Progress, Some(role));
    element.props.progress = Some(value);
    element
}
