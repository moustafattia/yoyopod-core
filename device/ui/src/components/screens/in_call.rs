use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::{Scene, SceneDefaults};

pub struct InCallProps {
    pub defaults: SceneDefaults,
    pub title: String,
    pub state: String,
    pub muted: bool,
}

pub fn props_from(snapshot: &RuntimeSnapshot, defaults: SceneDefaults) -> InCallProps {
    InCallProps {
        defaults,
        title: call_peer_name(snapshot),
        state: format!(
            "{}\n{}",
            snapshot.call.duration_text, snapshot.call.peer_address
        ),
        muted: snapshot.call.muted,
    }
}

pub fn scene(props: &InCallProps) -> Scene {
    super::common::call_scene(
        UiScreen::InCall,
        &props.defaults,
        props.title.clone(),
        props.state.clone(),
        props.muted,
    )
}

fn call_peer_name(snapshot: &RuntimeSnapshot) -> String {
    if snapshot.call.peer_name.trim().is_empty() {
        "Unknown".to_string()
    } else {
        snapshot.call.peer_name.clone()
    }
}
