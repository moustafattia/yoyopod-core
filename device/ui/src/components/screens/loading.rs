use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::{Modal, Scene, SceneDefaults};

pub struct LoadingProps {
    pub defaults: SceneDefaults,
    pub message: String,
}

pub fn props_from(snapshot: &RuntimeSnapshot, defaults: SceneDefaults) -> LoadingProps {
    LoadingProps {
        defaults,
        message: snapshot.overlay.message.clone(),
    }
}

pub fn scene(props: &LoadingProps) -> Scene {
    super::common::overlay_scene(
        UiScreen::Loading,
        &props.defaults,
        Modal::Loading {
            title: "Loading".to_string(),
            message: props.message.clone(),
        },
    )
}
