use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::Scene;

pub fn scene(snapshot: &RuntimeSnapshot) -> Scene {
    let mute = if snapshot.call.muted { "Muted" } else { "Live" };
    super::common::call_scene(
        UiScreen::InCall,
        call_peer_name(snapshot),
        format!(
            "{mute}\n{}\n{}",
            snapshot.call.duration_text, snapshot.call.peer_address
        ),
    )
}

fn call_peer_name(snapshot: &RuntimeSnapshot) -> String {
    if snapshot.call.peer_name.trim().is_empty() {
        "Unknown".to_string()
    } else {
        snapshot.call.peer_name.clone()
    }
}
