use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::{Modal, Scene, SceneDefaults};

pub struct ErrorProps {
    pub defaults: SceneDefaults,
    pub message: String,
}

pub fn props_from(snapshot: &RuntimeSnapshot, defaults: SceneDefaults) -> ErrorProps {
    ErrorProps {
        defaults,
        message: snapshot.overlay.error.clone(),
    }
}

pub fn scene(props: &ErrorProps) -> Scene {
    super::common::overlay_scene(
        UiScreen::Error,
        &props.defaults,
        Modal::Error {
            title: "Error".to_string(),
            message: props.message.clone(),
        },
    )
}
