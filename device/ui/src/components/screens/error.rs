use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::{Modal, Scene};

pub struct ErrorProps {
    pub message: String,
}

pub fn props_from(snapshot: &RuntimeSnapshot) -> ErrorProps {
    ErrorProps {
        message: snapshot.overlay.error.clone(),
    }
}

pub fn scene(props: &ErrorProps) -> Scene {
    super::common::overlay_scene(
        UiScreen::Error,
        Modal::Error {
            title: "Error".to_string(),
            message: props.message.clone(),
        },
    )
}
