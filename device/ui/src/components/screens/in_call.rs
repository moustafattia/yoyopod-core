use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::{Scene, SceneDefaults};

pub struct InCallProps {
    pub defaults: SceneDefaults,
    pub title: String,
    pub body: String,
}

pub fn props_from(snapshot: &RuntimeSnapshot, defaults: SceneDefaults) -> InCallProps {
    let mute = if snapshot.call.muted { "Muted" } else { "Live" };
    InCallProps {
        defaults,
        title: call_peer_name(snapshot),
        body: format!(
            "{mute}\n{}\n{}",
            snapshot.call.duration_text, snapshot.call.peer_address
        ),
    }
}

pub fn scene(props: &InCallProps) -> Scene {
    super::common::call_scene(
        UiScreen::InCall,
        &props.defaults,
        props.title.clone(),
        props.body.clone(),
    )
}

fn call_peer_name(snapshot: &RuntimeSnapshot) -> String {
    if snapshot.call.peer_name.trim().is_empty() {
        "Unknown".to_string()
    } else {
        snapshot.call.peer_name.clone()
    }
}
