use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::engine::Key;
use crate::scene::{ButtonModel, DeckItem, ItemRender, Scene};

pub struct TalkContactProps {
    pub actions: Vec<DeckItem>,
    pub focus: usize,
}

pub fn props_from(
    snapshot: &RuntimeSnapshot,
    focus: usize,
    selected_contact: Option<&ListItemSnapshot>,
) -> TalkContactProps {
    TalkContactProps {
        actions: actions(snapshot, selected_contact),
        focus,
    }
}

pub fn scene(props: &TalkContactProps) -> Scene {
    let mut scene = super::common::action_scene(UiScreen::TalkContact, props.focus);
    if let Some(deck) = scene.decks.first_mut() {
        deck.items = props.actions.clone();
    }
    scene.cursor = Some(crate::scene::Cursor::UnderlineDots {
        count: scene
            .decks
            .first()
            .map(|deck| deck.items.len())
            .unwrap_or(0),
        focus: props.focus,
    });
    scene
}

fn actions(
    snapshot: &RuntimeSnapshot,
    selected_contact: Option<&ListItemSnapshot>,
) -> Vec<DeckItem> {
    let contact = selected_contact.or_else(|| snapshot.call.contacts.first());
    let mut actions = vec![
        button("call", "Call", "call"),
        button("voice_note", "Voice Note", "voice_note"),
    ];
    if contact
        .and_then(|contact| snapshot.call.latest_voice_note_by_contact.get(&contact.id))
        .is_some_and(|note| !note.local_file_path.trim().is_empty())
    {
        actions.push(button("play_note", "Play Note", "play"));
    }
    actions
}

fn button(key: &'static str, title: &'static str, icon_key: &'static str) -> DeckItem {
    DeckItem {
        key: Key::Static(key),
        render: ItemRender::Button(ButtonModel {
            title: title.to_string(),
            icon_key: icon_key.to_string(),
        }),
    }
}
