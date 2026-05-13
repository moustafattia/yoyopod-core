use yoyopod_protocol::ui::UiScreen;

use crate::animation::Timeline;

use super::{Backdrop, Cursor, Deck, FxLayer, Modal, Stage};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scene {
    pub id: SceneId,
    pub backdrop: Backdrop,
    pub stage: Stage,
    pub decks: Vec<Deck>,
    pub cursor: Option<Cursor>,
    pub fx: FxLayer,
    pub modal: Option<Modal>,
    pub timelines: Vec<Timeline>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SceneId {
    pub screen: UiScreen,
    pub generation: u32,
}

impl SceneId {
    pub const fn new(screen: UiScreen) -> Self {
        Self {
            screen,
            generation: 0,
        }
    }
}
