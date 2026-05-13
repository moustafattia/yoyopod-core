use yoyopod_protocol::ui::UiScreen;

use crate::animation::presets;
use crate::scene::{
    Backdrop, Deck, DeckItemAnim, DeckKind, FxLayer, RegionId, Scene, SceneId, Stage,
};

pub fn scene(focus: usize) -> Scene {
    Scene {
        id: SceneId::new(UiScreen::Power),
        backdrop: Backdrop::Solid(0x2a2d35),
        stage: Stage::PagedDetail,
        decks: vec![Deck {
            kind: DeckKind::Page,
            region: RegionId::ListBody,
            items: Vec::new(),
            focus_index: focus,
            focus_policy: crate::router::FocusPolicy::Wrap,
            item_anim: DeckItemAnim::None,
            swap_anim: None,
            recycle_window: None,
        }],
        cursor: None,
        fx: FxLayer::default(),
        modal: None,
        timelines: vec![presets::scene_enter()],
    }
}
