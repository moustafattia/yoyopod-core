#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerSlot {
    Backdrop,
    Stage,
    Decks,
    Cursor,
    Fx,
    Hud,
    Modal,
}

pub const LAYER_ORDER: [LayerSlot; 7] = [
    LayerSlot::Backdrop,
    LayerSlot::Stage,
    LayerSlot::Decks,
    LayerSlot::Cursor,
    LayerSlot::Fx,
    LayerSlot::Hud,
    LayerSlot::Modal,
];

impl LayerSlot {
    pub const fn is_scene_owned(self) -> bool {
        matches!(
            self,
            Self::Backdrop | Self::Stage | Self::Decks | Self::Cursor | Self::Fx
        )
    }

    pub const fn is_graph_overlay(self) -> bool {
        matches!(self, Self::Hud | Self::Modal)
    }
}
