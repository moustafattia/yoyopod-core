use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::{Scene, SceneDefaults};

pub struct ContactsProps {
    pub defaults: SceneDefaults,
    pub items: Vec<ListItemSnapshot>,
    pub focus: usize,
}

pub fn props_from(
    snapshot: &RuntimeSnapshot,
    focus: usize,
    defaults: SceneDefaults,
) -> ContactsProps {
    ContactsProps {
        defaults,
        items: snapshot.call.contacts.clone(),
        focus,
    }
}

pub fn scene(props: &ContactsProps) -> Scene {
    super::common::list_scene(
        UiScreen::Contacts,
        &props.defaults,
        &props.items,
        props.focus,
        FocusPolicy::Clamp,
    )
}
