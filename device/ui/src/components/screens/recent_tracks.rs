use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::Scene;

pub fn scene(snapshot: &RuntimeSnapshot, focus: usize) -> Scene {
    super::common::list_scene(
        UiScreen::RecentTracks,
        &snapshot.music.recent_tracks,
        focus,
        FocusPolicy::Clamp,
    )
}
