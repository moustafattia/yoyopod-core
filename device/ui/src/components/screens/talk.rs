use yoyopod_protocol::ui::UiScreen;

use crate::engine::Key;
use crate::scene::{CardModel, DeckItem, ItemRender, Scene};

pub struct TalkProps {
    pub cards: Vec<DeckItem>,
    pub focus: usize,
}

pub fn props_from(focus: usize) -> TalkProps {
    TalkProps {
        cards: vec![
            card("contacts", "Contacts", "Call someone", "contact", 0x00d4ff),
            card(
                "call_history",
                "History",
                "Recent calls",
                "history",
                0x74d7ff,
            ),
            card("voice_note", "Voice Note", "Record a note", "mic", 0xc79bff),
        ],
        focus,
    }
}

pub fn scene(props: &TalkProps) -> Scene {
    let mut scene = super::common::hero_scene(UiScreen::Talk, 0x00d4ff, 3, props.focus);
    if let Some(deck) = scene.decks.first_mut() {
        deck.items = props.cards.clone();
    }
    scene
}

fn card(
    key: &'static str,
    title: &'static str,
    subtitle: &'static str,
    icon_key: &'static str,
    accent: u32,
) -> DeckItem {
    DeckItem {
        key: Key::Static(key),
        render: ItemRender::Card(CardModel {
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            icon_key: icon_key.to_string(),
            accent,
        }),
    }
}
