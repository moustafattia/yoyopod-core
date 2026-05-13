use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::animation::presets;
use crate::scene::{Backdrop, FxLayer, Scene, SceneId, Stage};

pub fn scene(_snapshot: &RuntimeSnapshot) -> Scene {
    Scene {
        id: SceneId::new(UiScreen::NowPlaying),
        backdrop: Backdrop::Solid(0x2a2d35),
        stage: Stage::NowPlayingPanel,
        decks: Vec::new(),
        cursor: None,
        fx: FxLayer::default(),
        modal: None,
        timelines: vec![presets::scene_enter()],
    }
}
