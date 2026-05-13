use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::Scene;

pub fn scene(snapshot: &RuntimeSnapshot) -> Scene {
    super::common::call_scene(
        UiScreen::IncomingCall,
        call_peer_name(snapshot),
        format!("Incoming Call\n{}", snapshot.call.peer_address),
    )
}

fn call_peer_name(snapshot: &RuntimeSnapshot) -> String {
    if snapshot.call.peer_name.trim().is_empty() {
        "Unknown".to_string()
    } else {
        snapshot.call.peer_name.clone()
    }
}
