use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::{Modal, Scene, SceneDefaults};

pub struct LoadingProps {
    pub message: String,
}

pub fn props_from(snapshot: &RuntimeSnapshot) -> LoadingProps {
    LoadingProps {
        message: snapshot.overlay.message.clone(),
    }
}

pub fn scene(props: &LoadingProps, defaults: &SceneDefaults) -> Scene {
    super::common::overlay_scene(
        UiScreen::Loading,
        defaults,
        Modal::Loading {
            title: "Loading".to_string(),
            message: props.message.clone(),
        },
    )
}
