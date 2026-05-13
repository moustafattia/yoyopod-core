use crate::scene::RegionId;

use super::{ClockSource, LoopMode, Timeline, TimelineId};

pub fn breathe_around(_region: RegionId) -> Timeline {
    Timeline {
        id: TimelineId(0),
        clock: ClockSource::GlobalTime,
        tracks: Vec::new(),
        loop_mode: LoopMode::Loop,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn scene_enter() -> Timeline {
    Timeline {
        id: TimelineId(1),
        clock: ClockSource::SceneTime,
        tracks: Vec::new(),
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn stagger_enter() -> Timeline {
    Timeline {
        id: TimelineId(2),
        clock: ClockSource::SceneTime,
        tracks: Vec::new(),
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}
