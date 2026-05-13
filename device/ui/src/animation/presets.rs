use crate::scene::RegionId;

use super::{
    ActorRef, AnimatableProp, AnimatableValue, ClockSource, Easing, EventId, Keyframe, LoopMode,
    Timeline, TimelineId, TimelineRef, Track,
};

pub const BREATHE_TIMELINE_ID: TimelineId = TimelineId(10);
pub const SCENE_ENTER_TIMELINE_ID: TimelineId = TimelineId(1);
pub const STAGGER_ENTER_TIMELINE_ID: TimelineId = TimelineId(2);
pub const PULSE_ONE_SHOT_TIMELINE_ID: TimelineId = TimelineId(3);
pub const SLIDE_IN_FROM_RIGHT_TIMELINE_ID: TimelineId = TimelineId(4);
pub const PROGRESS_SWEEP_TIMELINE_ID: TimelineId = TimelineId(5);
pub const SELECTION_SNAP_TIMELINE_ID: TimelineId = TimelineId(6);

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
        tracks: vec![
            Track {
                target: ActorRef::Screen,
                property: AnimatableProp::Opacity,
                keyframes: vec![
                    Keyframe {
                        at_ms: 0,
                        value: AnimatableValue::U8(0),
                    },
                    Keyframe {
                        at_ms: 220,
                        value: AnimatableValue::U8(255),
                    },
                ],
                easing: Easing::EaseOut,
            },
            Track {
                target: ActorRef::Screen,
                property: AnimatableProp::Y,
                keyframes: vec![
                    Keyframe {
                        at_ms: 0,
                        value: AnimatableValue::I32(8),
                    },
                    Keyframe {
                        at_ms: 220,
                        value: AnimatableValue::I32(0),
                    },
                ],
                easing: Easing::EaseOut,
            },
        ],
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn stagger_enter() -> Timeline {
    let tracks = (0..4)
        .map(|index| Track {
            target: ActorRef::DeckItem { deck: 0, index },
            property: AnimatableProp::Opacity,
            keyframes: vec![
                Keyframe {
                    at_ms: 40 * index as u32,
                    value: AnimatableValue::U8(0),
                },
                Keyframe {
                    at_ms: 160 + 40 * index as u32,
                    value: AnimatableValue::U8(255),
                },
            ],
            easing: Easing::EaseOut,
        })
        .collect();
    Timeline {
        id: STAGGER_ENTER_TIMELINE_ID,
        clock: ClockSource::SceneTime,
        tracks,
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn pulse_one_shot(actor: ActorRef) -> Timeline {
    Timeline {
        id: PULSE_ONE_SHOT_TIMELINE_ID,
        clock: ClockSource::EventTime(EventId(3)),
        tracks: vec![
            Track {
                target: actor,
                property: AnimatableProp::Opacity,
                keyframes: vec![
                    Keyframe {
                        at_ms: 0,
                        value: AnimatableValue::U8(192),
                    },
                    Keyframe {
                        at_ms: 600,
                        value: AnimatableValue::U8(0),
                    },
                ],
                easing: Easing::EaseOut,
            },
            Track {
                target: actor,
                property: AnimatableProp::Scale,
                keyframes: vec![
                    Keyframe {
                        at_ms: 0,
                        value: AnimatableValue::I32(920),
                    },
                    Keyframe {
                        at_ms: 600,
                        value: AnimatableValue::I32(1120),
                    },
                ],
                easing: Easing::EaseOut,
            },
        ],
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn slide_in_from_right() -> Timeline {
    Timeline {
        id: SLIDE_IN_FROM_RIGHT_TIMELINE_ID,
        clock: ClockSource::SceneTime,
        tracks: vec![Track {
            target: ActorRef::Screen,
            property: AnimatableProp::X,
            keyframes: vec![
                Keyframe {
                    at_ms: 0,
                    value: AnimatableValue::I32(28),
                },
                Keyframe {
                    at_ms: 220,
                    value: AnimatableValue::I32(0),
                },
            ],
            easing: Easing::EaseOut,
        }],
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn progress_sweep(from: i32, to: i32) -> Timeline {
    Timeline {
        id: PROGRESS_SWEEP_TIMELINE_ID,
        clock: ClockSource::EventTime(EventId(5)),
        tracks: vec![Track {
            target: ActorRef::Region(RegionId::Progress),
            property: AnimatableProp::ProgressPermille,
            keyframes: vec![
                Keyframe {
                    at_ms: 0,
                    value: AnimatableValue::I32(from),
                },
                Keyframe {
                    at_ms: 360,
                    value: AnimatableValue::I32(to),
                },
            ],
            easing: Easing::EaseInOut,
        }],
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn selection_snap(to_index: usize) -> Timeline {
    Timeline {
        id: SELECTION_SNAP_TIMELINE_ID,
        clock: ClockSource::EventTime(EventId(6)),
        tracks: vec![Track {
            target: ActorRef::Cursor,
            property: AnimatableProp::SelectionOffset,
            keyframes: vec![
                Keyframe {
                    at_ms: 0,
                    value: AnimatableValue::I32(0),
                },
                Keyframe {
                    at_ms: 120,
                    value: AnimatableValue::I32((to_index as i32) * 1_000),
                },
            ],
            easing: Easing::EaseOut,
        }],
        loop_mode: LoopMode::Once,
        on_complete: None,
        started_ms: 0,
    }
}

pub fn timeline_for_ref(reference: TimelineRef) -> Timeline {
    match reference.0 {
        SCENE_ENTER_TIMELINE_ID => scene_enter(),
        STAGGER_ENTER_TIMELINE_ID => stagger_enter(),
        SLIDE_IN_FROM_RIGHT_TIMELINE_ID => slide_in_from_right(),
        TimelineId(value) => panic!("unknown route timeline id {value}"),
    }
}
