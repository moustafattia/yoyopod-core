use crate::engine::Element;
use crate::scene::FooterBar;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterBarProps {
    pub text: String,
    pub accent: Option<u32>,
}

pub fn footer_bar(props: &FooterBarProps) -> Element {
    FooterBar {
        text: props.text.clone(),
        accent: props.accent,
    }
    .element()
}
