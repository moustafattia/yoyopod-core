use crate::components::primitives::{container, label};
use crate::engine::Element;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterBarProps {
    pub text: String,
    pub accent: Option<u32>,
}

pub fn footer_bar(props: &FooterBarProps) -> Element {
    let mut text = label("footer_label").text(&props.text);
    if let Some(accent) = props.accent {
        text = text.accent(accent);
    }
    container("footer_bar").child(text)
}
