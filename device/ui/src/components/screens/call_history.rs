use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::Scene;

pub struct CallHistoryProps {
    pub items: Vec<ListItemSnapshot>,
    pub focus: usize,
}

pub fn props_from(snapshot: &RuntimeSnapshot, focus: usize) -> CallHistoryProps {
    CallHistoryProps {
        items: snapshot.call.history.clone(),
        focus,
    }
}

pub fn scene(props: &CallHistoryProps) -> Scene {
    super::common::list_scene(
        UiScreen::CallHistory,
        &props.items,
        props.focus,
        FocusPolicy::Clamp,
    )
}
