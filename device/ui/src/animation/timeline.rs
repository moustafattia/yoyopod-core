use super::{AnimatableProp, AnimatableValue, ClockSource, Easing};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimelineId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TimelineRef(pub TimelineId);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TrackIndex(pub usize);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeline {
    pub id: TimelineId,
    pub clock: ClockSource,
    pub tracks: Vec<Track>,
    pub loop_mode: LoopMode,
    pub on_complete: Option<OnCompleteAction>,
    pub started_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Track {
    pub target: super::ActorRef,
    pub property: AnimatableProp,
    pub keyframes: Vec<Keyframe>,
    pub easing: Easing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Keyframe {
    pub at_ms: u32,
    pub value: AnimatableValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopMode {
    Once,
    Loop,
    PingPong,
    RepeatN(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnCompleteAction {
    MarkDirty,
}
