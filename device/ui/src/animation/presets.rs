use crate::scene::RegionId;

use super::{
    ActorRef, AnimatableProp, AnimatableValue, ClockSource, Easing, Keyframe, LoopMode, Timeline,
    TimelineId, TimelineRef, Track,
};

pub const BREATHE_TIMELINE_ID: TimelineId = TimelineId(10);
pub const SCENE_ENTER_TIMELINE_ID: TimelineId = TimelineId(1);
pub const STAGGER_ENTER_TIMELINE_ID: TimelineId = TimelineId(2);

pub fn breathe_around(region: RegionId) -> Timeline {
    Timeline {
        id: BREATHE_TIMELINE_ID,
        clock: ClockSource::GlobalTime,
        tracks: vec![Track {
            target: ActorRef::Region(region),
            property: AnimatableProp::Opacity,
            keyframes: vec![
                Keyframe {
                    at_ms: 0,
                    value: AnimatableValue::U8(64),
                },
                Keyframe {
                    at_ms: 700,
                    value: AnimatableValue::U8(128),
                },
                Keyframe {
                    at_ms: 1_400,
                    value: AnimatableValue::U8(64),
                },
            ],
            easing: Easing::EaseInOut,
        }],
        loop_mode: LoopMode::Loop,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn scene_enter() -> Timeline {
    Timeline {
        id: SCENE_ENTER_TIMELINE_ID,
        clock: ClockSource::SceneTime,
        tracks: Vec::new(),
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn stagger_enter() -> Timeline {
    Timeline {
        id: STAGGER_ENTER_TIMELINE_ID,
        clock: ClockSource::SceneTime,
        tracks: Vec::new(),
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn timeline_for_ref(reference: TimelineRef) -> Timeline {
    match reference.0 {
        SCENE_ENTER_TIMELINE_ID => scene_enter(),
        STAGGER_ENTER_TIMELINE_ID => stagger_enter(),
        TimelineId(value) => panic!("unknown route timeline id {value}"),
    }
}
