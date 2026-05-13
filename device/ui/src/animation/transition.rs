use yoyopod_protocol::ui::AnimationRequest;

use yoyopod_protocol::ui::UiScreen;

use super::{
    ActorRef, AnimatableProp, AnimatableValue, ClockSource, EventId, Keyframe, LoopMode, Timeline,
    TimelineId, Track,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransitionTarget {
    Screen(UiScreen),
    Selection { screen: UiScreen, index: usize },
    Runtime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionProperty {
    Opacity,
    OffsetY,
    ScalePermille,
    SelectionOffset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transition {
    pub id: String,
    pub target: TransitionTarget,
    pub property: TransitionProperty,
    pub easing: Easing,
    pub from: i32,
    pub to: i32,
    pub duration_ms: u64,
    pub started_at_ms: u64,
}

impl Transition {
    pub fn from_request(
        request: AnimationRequest,
        screen: UiScreen,
        focus_index: usize,
        started_at_ms: u64,
    ) -> Self {
        let id = if request.transition_id.trim().is_empty() {
            "screen_enter".to_string()
        } else {
            request.transition_id
        };
        let target = if id.starts_with("selection") {
            TransitionTarget::Selection {
                screen,
                index: focus_index,
            }
        } else if id.starts_with("runtime") {
            TransitionTarget::Runtime
        } else {
            TransitionTarget::Screen(screen)
        };
        let property = if id.contains("selection") {
            TransitionProperty::SelectionOffset
        } else if id.contains("scale") {
            TransitionProperty::ScalePermille
        } else if id.contains("fade") {
            TransitionProperty::Opacity
        } else {
            TransitionProperty::OffsetY
        };
        let (from, to) = match property {
            TransitionProperty::Opacity => (0, 255),
            TransitionProperty::ScalePermille => (960, 1000),
            TransitionProperty::OffsetY | TransitionProperty::SelectionOffset => (8, 0),
        };
        Self {
            id,
            target,
            property,
            easing: Easing::EaseOut,
            from,
            to,
            duration_ms: request.duration_ms.max(120),
            started_at_ms,
        }
    }

    pub fn is_complete(&self, now_ms: u64) -> bool {
        now_ms.saturating_sub(self.started_at_ms) >= self.duration_ms
    }

    pub fn timeline(&self) -> Timeline {
        Timeline {
            id: TimelineId(timeline_id(&self.id)),
            clock: ClockSource::EventTime(EventId(event_id(&self.id))),
            tracks: vec![Track {
                target: actor_ref(self.target.clone()),
                property: animatable_property(self.property),
                keyframes: vec![
                    Keyframe {
                        at_ms: 0,
                        value: animatable_value(self.property, self.from),
                    },
                    Keyframe {
                        at_ms: self.duration_ms.min(u64::from(u32::MAX)) as u32,
                        value: animatable_value(self.property, self.to),
                    },
                ],
                easing: easing(self.easing),
            }],
            loop_mode: LoopMode::Once,
            on_complete: None,
            started_ms: self.started_at_ms,
        }
    }
}

fn actor_ref(target: TransitionTarget) -> ActorRef {
    match target {
        TransitionTarget::Screen(_) | TransitionTarget::Runtime => ActorRef::Screen,
        TransitionTarget::Selection { .. } => ActorRef::Cursor,
    }
}

fn animatable_property(property: TransitionProperty) -> AnimatableProp {
    match property {
        TransitionProperty::Opacity => AnimatableProp::Opacity,
        TransitionProperty::OffsetY => AnimatableProp::Y,
        TransitionProperty::ScalePermille => AnimatableProp::Scale,
        TransitionProperty::SelectionOffset => AnimatableProp::SelectionOffset,
    }
}

fn animatable_value(property: TransitionProperty, value: i32) -> AnimatableValue {
    match property {
        TransitionProperty::Opacity => AnimatableValue::U8(value.clamp(0, 255) as u8),
        TransitionProperty::OffsetY
        | TransitionProperty::ScalePermille
        | TransitionProperty::SelectionOffset => AnimatableValue::I32(value),
    }
}

fn easing(easing: Easing) -> super::Easing {
    match easing {
        Easing::Linear => super::Easing::Linear,
        Easing::EaseOut => super::Easing::EaseOut,
        Easing::EaseInOut => super::Easing::EaseInOut,
    }
}

fn timeline_id(id: &str) -> u32 {
    10_000 + (stable_hash(id) % 50_000) as u32
}

fn event_id(id: &str) -> u64 {
    10_000 + stable_hash(id)
}

fn stable_hash(id: &str) -> u64 {
    id.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        hash.wrapping_mul(0x100_0000_01b3)
            .wrapping_add(u64::from(byte))
    })
}
