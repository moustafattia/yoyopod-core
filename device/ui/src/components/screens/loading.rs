use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::{Modal, Scene};

pub fn scene(snapshot: &RuntimeSnapshot) -> Scene {
    super::common::overlay_scene(
        UiScreen::Loading,
        Modal::Loading {
            title: "Loading".to_string(),
            message: snapshot.overlay.message.clone(),
        },
    )
}
