use crate::engine::Element;
use crate::engine::Key;
use crate::render_contract::ElementKind;
use crate::roles;
use crate::scene::RegionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FooterBarProps {
    pub text: String,
    pub accent: Option<u32>,
}

pub fn footer_bar(props: &FooterBarProps) -> Element {
    let mut label = Element::new(ElementKind::Label, Some(roles::FOOTER_LABEL))
        .key(Key::Static("footer_label"))
        .text(&props.text);
    if let Some(accent) = props.accent {
        label = label.accent(accent);
    }
    Element::new(ElementKind::Container, Some(roles::FOOTER_BAR))
        .key(Key::Static("footer_bar"))
        .region(RegionId::Footer)
        .child(label)
}
