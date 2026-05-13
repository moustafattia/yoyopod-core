use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::animation::presets;
use crate::router::FocusPolicy;
use crate::scene::{
    Backdrop, Cursor, Deck, DeckItemAnim, DeckKind, FxLayer, RegionId, Scene, SceneId, Stage,
};

pub struct HubProps {
    pub card_count: usize,
    pub selected_index: usize,
    pub accent: u32,
}

pub fn props_from(snapshot: &RuntimeSnapshot, focus: usize) -> HubProps {
    let cards = &snapshot.hub.cards;
    let selected = cards.get(focus).or_else(|| cards.first());
    HubProps {
        card_count: cards.len(),
        selected_index: focus,
        accent: selected.map(|card| card.accent).unwrap_or(0x3ddd53),
    }
}

pub fn scene(props: &HubProps) -> Scene {
    Scene {
        id: SceneId::new(UiScreen::Hub),
        backdrop: Backdrop::AccentDrift {
            accent: props.accent,
            speed_ms: 800,
        },
        stage: Stage::CenteredHeroIcon,
        decks: vec![Deck {
            kind: DeckKind::CardRow,
            region: RegionId::HeroIcon,
            items: Vec::new(),
            focus_index: props.selected_index,
            focus_policy: FocusPolicy::Wrap,
            item_anim: DeckItemAnim::ScaleOnFocus {
                from_permille: 960,
                to_permille: 1000,
            },
            swap_anim: None,
            recycle_window: Some(3),
        }],
        cursor: Some(Cursor::UnderlineDots {
            count: props.card_count,
            focus: props.selected_index,
        }),
        fx: FxLayer::default(),
        modal: None,
        timelines: vec![presets::scene_enter()],
    }
}
