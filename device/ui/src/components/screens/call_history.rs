use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::{Scene, SceneDefaults};

pub struct CallHistoryProps {
    pub defaults: SceneDefaults,
    pub items: Vec<ListItemSnapshot>,
    pub focus: usize,
}

pub fn props_from(
    snapshot: &RuntimeSnapshot,
    focus: usize,
    defaults: SceneDefaults,
) -> CallHistoryProps {
    CallHistoryProps {
        defaults,
        items: snapshot.call.history.clone(),
        focus,
    }
}

pub fn scene(props: &CallHistoryProps) -> Scene {
    super::common::list_scene(
        UiScreen::CallHistory,
        &props.defaults,
        &props.items,
        props.focus,
        FocusPolicy::Clamp,
    )
}
