use crate::components::primitives::{container, label};
use crate::engine::Element;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChipProps {
    pub role: &'static str,
    pub label_role: &'static str,
    pub text: String,
    pub accent: Option<u32>,
}

pub fn chip(props: &ChipProps) -> Element {
    let mut root = container(props.role);
    if let Some(accent) = props.accent {
        root = root.accent(accent);
    }
    root.child(label(props.label_role).text(&props.text))
}
