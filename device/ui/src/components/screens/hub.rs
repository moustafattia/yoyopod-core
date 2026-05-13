use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::engine::Key;
use crate::router::FocusPolicy;
use crate::scene::{
    CardModel, Cursor, Deck, DeckItem, DeckItemAnim, DeckKind, ItemRender, RegionId, Scene,
    SceneDefaults, SceneId,
};

pub struct HubProps {
    pub defaults: SceneDefaults,
    pub card_count: usize,
    pub selected_index: usize,
    pub accent: u32,
    pub cards: Vec<DeckItem>,
}

pub fn props_from(snapshot: &RuntimeSnapshot, focus: usize, defaults: SceneDefaults) -> HubProps {
    let cards = &snapshot.hub.cards;
    let selected = cards.get(focus).or_else(|| cards.first());
    HubProps {
        defaults,
        card_count: cards.len(),
        selected_index: focus,
        accent: selected.map(|card| card.accent).unwrap_or(0x3ddd53),
        cards: cards
            .iter()
            .map(|card| DeckItem {
                key: Key::String(card.key.clone()),
                render: ItemRender::Card(CardModel {
                    title: card.title.clone(),
                    subtitle: card.subtitle.clone(),
                    icon_key: card.key.clone(),
                    accent: card.accent,
                }),
            })
            .collect(),
    }
}

pub fn scene(props: &HubProps) -> Scene {
    Scene {
        id: SceneId::new(UiScreen::Hub),
        backdrop: props.defaults.backdrop(props.accent),
        stage: props.defaults.stage,
        decks: vec![Deck {
            kind: DeckKind::CardRow,
            region: RegionId::HeroIcon,
            items: props.cards.clone(),
            focus_index: props.selected_index,
            focus_policy: FocusPolicy::Wrap,
            item_anim: DeckItemAnim::BreatheWhenFocused,
            swap_anim: None,
            recycle_window: Some(3),
        }],
        cursor: Some(Cursor::UnderlineDots {
            count: props.card_count,
            focus: props.selected_index,
        }),
        fx: props.defaults.fx_layer(props.accent),
        modal: None,
        timelines: props.defaults.fx_timelines(),
    }
}
