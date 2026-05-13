use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::Scene;

pub struct IncomingCallProps {
    pub title: String,
    pub body: String,
}

pub fn props_from(snapshot: &RuntimeSnapshot) -> IncomingCallProps {
    IncomingCallProps {
        title: call_peer_name(snapshot),
        body: format!("Incoming Call\n{}", snapshot.call.peer_address),
    }
}

pub fn scene(props: &IncomingCallProps) -> Scene {
    super::common::call_scene(
        UiScreen::IncomingCall,
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
