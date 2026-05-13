use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::{Modal, Scene};

pub struct LoadingProps {
    pub message: String,
}

pub fn props_from(snapshot: &RuntimeSnapshot) -> LoadingProps {
    LoadingProps {
        message: snapshot.overlay.message.clone(),
    }
}

pub fn scene(props: &LoadingProps) -> Scene {
    super::common::overlay_scene(
        UiScreen::Loading,
        Modal::Loading {
            title: "Loading".to_string(),
            message: props.message.clone(),
        },
    )
}
