use crate::components::primitives::{container, label};
use crate::engine::Element;
use crate::scene::roles;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallPanelProps {
    pub title: String,
    pub state: String,
    pub muted: bool,
}

pub fn call_panel(props: &CallPanelProps) -> Element {
    container(roles::CALL_PANEL)
        .child(label(roles::CALL_TITLE).text(&props.title))
        .child(label(roles::CALL_STATE_LABEL).text(&props.state))
        .child(label(roles::CALL_MUTE_LABEL).text(if props.muted { "Muted" } else { "" }))
}
