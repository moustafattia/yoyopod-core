use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::Scene;

pub struct OutgoingCallProps {
    pub title: String,
    pub body: String,
}

pub fn props_from(snapshot: &RuntimeSnapshot) -> OutgoingCallProps {
    OutgoingCallProps {
        title: call_peer_name(snapshot),
        body: format!("Dialing\n{}", snapshot.call.peer_address),
    }
}

pub fn scene(props: &OutgoingCallProps) -> Scene {
    super::common::call_scene(
        UiScreen::OutgoingCall,
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
