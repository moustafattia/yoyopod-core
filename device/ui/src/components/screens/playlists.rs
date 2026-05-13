use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::scene::{FocusPolicy, Scene, SceneDefaults};

pub struct PlaylistsProps {
    pub defaults: SceneDefaults,
    pub items: Vec<ListItemSnapshot>,
    pub focus: usize,
}

pub fn props_from(
    snapshot: &RuntimeSnapshot,
    focus: usize,
    defaults: SceneDefaults,
) -> PlaylistsProps {
    PlaylistsProps {
        defaults,
        items: snapshot.music.playlists.clone(),
        focus,
    }
}

pub fn scene(props: &PlaylistsProps) -> Scene {
    super::common::list_scene(
        UiScreen::Playlists,
        &props.defaults,
        &props.items,
        props.focus,
        FocusPolicy::Clamp,
    )
}
