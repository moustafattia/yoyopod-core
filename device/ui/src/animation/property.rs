use crate::scene::{FxLayerId, RegionId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatableProp {
    Opacity,
    X,
    Y,
    Width,
    Height,
    Scale,
    AccentMix,
    BorderWidth,
    ShadowRadius,
    SelectionOffset,
    ProgressPermille,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimatableValue {
    I32(i32),
    U8(u8),
    Rgb(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorRef {
    Screen,
    Region(RegionId),
    DeckItem { deck: usize, index: usize },
    FxNode { layer: FxLayerId, index: usize },
    Cursor,
}
