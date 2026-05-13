use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::{Modal, Scene};

pub fn scene(snapshot: &RuntimeSnapshot) -> Scene {
    super::common::overlay_scene(
        UiScreen::Error,
        Modal::Error {
            title: "Error".to_string(),
            message: snapshot.overlay.error.clone(),
        },
    )
}
