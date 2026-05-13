use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::{Scene, SceneDefaults};

pub struct PlaylistsProps {
    pub items: Vec<ListItemSnapshot>,
    pub focus: usize,
}

pub fn props_from(snapshot: &RuntimeSnapshot, focus: usize) -> PlaylistsProps {
    PlaylistsProps {
        items: snapshot.music.playlists.clone(),
        focus,
    }
}

pub fn scene(props: &PlaylistsProps, defaults: &SceneDefaults) -> Scene {
    super::common::list_scene(
        UiScreen::Playlists,
        defaults,
        &props.items,
        props.focus,
        FocusPolicy::Clamp,
    )
}
