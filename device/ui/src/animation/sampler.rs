use super::{ActorRef, AnimatableProp, AnimatableValue, Timeline};

#[derive(Debug, Clone, Copy)]
pub struct TimelineSampler<'a> {
    timelines: &'a [Timeline],
    now_ms: u64,
    global_ms: u64,
}

impl<'a> TimelineSampler<'a> {
    pub const fn new(timelines: &'a [Timeline], now_ms: u64, global_ms: u64) -> Self {
        Self {
            timelines,
            now_ms,
            global_ms,
        }
    }

    pub const fn empty() -> Self {
        Self {
            timelines: &[],
            now_ms: 0,
            global_ms: 0,
        }
    }

    pub fn value(&self, _target: ActorRef, _property: AnimatableProp) -> Option<AnimatableValue> {
        let _ = (self.timelines, self.now_ms, self.global_ms);
        None
    }

    pub fn is_animating(&self, _target: ActorRef) -> bool {
        !self.timelines.is_empty()
    }
}
