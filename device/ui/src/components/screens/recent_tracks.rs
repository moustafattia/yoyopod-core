use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::{Scene, SceneDefaults};

pub struct RecentTracksProps {
    pub items: Vec<ListItemSnapshot>,
    pub focus: usize,
}

pub fn props_from(snapshot: &RuntimeSnapshot, focus: usize) -> RecentTracksProps {
    RecentTracksProps {
        items: snapshot.music.recent_tracks.clone(),
        focus,
    }
}

pub fn scene(props: &RecentTracksProps, defaults: &SceneDefaults) -> Scene {
    super::common::list_scene(
        UiScreen::RecentTracks,
        defaults,
        &props.items,
        props.focus,
        FocusPolicy::Clamp,
    )
}
