use crate::components::primitives::{container, image};
use crate::engine::Element;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IconHaloProps {
    pub halo_role: &'static str,
    pub icon_role: &'static str,
    pub icon_key: String,
    pub accent: u32,
}

pub fn icon_halo(props: &IconHaloProps) -> Element {
    container(props.halo_role).accent(props.accent).child(
        image(props.icon_role)
            .icon(&props.icon_key)
            .accent(props.accent),
    )
}
