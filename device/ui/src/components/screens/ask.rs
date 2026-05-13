use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::engine::Key;
use crate::scene::{CardModel, DeckItem, ItemRender, Scene};

pub fn scene(snapshot: &RuntimeSnapshot, focus: usize) -> Scene {
    let mut scene = super::common::hero_scene(UiScreen::Ask, 0xc79bff, 1, focus);
    if let Some(deck) = scene.decks.first_mut() {
        deck.items = vec![DeckItem {
            key: Key::Static("ask"),
            render: ItemRender::Card(CardModel {
                title: snapshot.voice.headline.clone(),
                subtitle: snapshot.voice.body.clone(),
                icon_key: "ask".to_string(),
                accent: 0xc79bff,
            }),
        }];
    }
    scene
}
