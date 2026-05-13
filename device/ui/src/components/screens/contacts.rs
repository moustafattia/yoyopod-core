use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::{Scene, SceneDefaults};

pub struct ContactsProps {
    pub items: Vec<ListItemSnapshot>,
    pub focus: usize,
}

pub fn props_from(snapshot: &RuntimeSnapshot, focus: usize) -> ContactsProps {
    ContactsProps {
        items: snapshot.call.contacts.clone(),
        focus,
    }
}

pub fn scene(props: &ContactsProps, defaults: &SceneDefaults) -> Scene {
    super::common::list_scene(
        UiScreen::Contacts,
        defaults,
        &props.items,
        props.focus,
        FocusPolicy::Clamp,
    )
}
