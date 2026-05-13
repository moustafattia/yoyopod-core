use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::Scene;

pub fn scene(snapshot: &RuntimeSnapshot, focus: usize) -> Scene {
    super::common::list_scene(
        UiScreen::Contacts,
        &snapshot.call.contacts,
        focus,
        FocusPolicy::Clamp,
    )
}
