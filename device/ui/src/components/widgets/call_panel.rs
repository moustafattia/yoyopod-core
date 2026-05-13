use crate::components::primitives::{container, label};
use crate::engine::Element;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallPanelProps {
    pub title: String,
    pub state: String,
    pub muted: bool,
}

pub fn call_panel(props: &CallPanelProps) -> Element {
    container("call_panel")
        .child(label("call_title").text(&props.title))
        .child(label("call_state_label").text(&props.state))
        .child(label("call_mute_label").text(if props.muted { "Muted" } else { "" }))
}
