use super::{
    ActorRef, AnimatableProp, AnimatableValue, ClockSource, Easing, LoopMode, Timeline,
    TimelineRef, TrackIndex,
};

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

    pub fn value(&self, target: ActorRef, property: AnimatableProp) -> Option<AnimatableValue> {
        self.timelines
            .iter()
            .flat_map(|timeline| {
                timeline
                    .tracks
                    .iter()
                    .filter(move |track| track.target == target && track.property == property)
                    .map(move |track| (timeline, track))
            })
            .find_map(|(timeline, track)| {
                let elapsed = self.elapsed_ms(timeline);
                sample_keyframes(&track.keyframes, track.easing, timeline.loop_mode, elapsed)
            })
    }

    pub fn slot_value(
        &self,
        timeline_ref: TimelineRef,
        track_index: TrackIndex,
    ) -> Option<(AnimatableProp, AnimatableValue)> {
        let timeline = self
            .timelines
            .iter()
            .find(|timeline| timeline.id == timeline_ref.0)?;
        let track = timeline.tracks.get(track_index.0)?;
        let value = sample_keyframes(
            &track.keyframes,
            track.easing,
            timeline.loop_mode,
            self.elapsed_ms(timeline),
        )?;
        Some((track.property, value))
    }

    pub fn quantized_signature(&self) -> Option<u64> {
        let mut signature = 0xcbf2_9ce4_8422_2325u64;
        let mut sampled = false;
        for timeline in self.timelines {
            for index in 0..timeline.tracks.len() {
                let Some((property, value)) =
                    self.slot_value(TimelineRef(timeline.id), TrackIndex(index))
                else {
                    continue;
                };
                sampled = true;
                signature = mix_signature(signature, property_signature(property));
                signature = mix_signature(signature, value_signature(value));
            }
        }
        sampled.then_some(signature)
    }

    pub fn is_animating(&self, _target: ActorRef) -> bool {
        !self.timelines.is_empty()
    }

    fn elapsed_ms(&self, timeline: &Timeline) -> u32 {
        let base = match timeline.clock {
            ClockSource::GlobalTime => self.global_ms,
            ClockSource::SceneTime | ClockSource::EventTime(_) => {
                self.now_ms.saturating_sub(timeline.started_ms)
            }
        };
        base.min(u64::from(u32::MAX)) as u32
    }
}

fn sample_keyframes(
    keyframes: &[super::Keyframe],
    easing: Easing,
    loop_mode: LoopMode,
    elapsed_ms: u32,
) -> Option<AnimatableValue> {
    let first = keyframes.first()?;
    let last = keyframes.last()?;
    if keyframes.len() == 1 {
        return Some(first.value);
    }

    let duration = last.at_ms.max(1);
    let t_ms = normalize_elapsed(elapsed_ms, duration, loop_mode);
    if t_ms <= first.at_ms {
        return Some(first.value);
    }
    if t_ms >= last.at_ms {
        return Some(last.value);
    }

    let (from, to) = keyframes.windows(2).find_map(|pair| {
        let from = pair[0];
        let to = pair[1];
        (t_ms >= from.at_ms && t_ms <= to.at_ms).then_some((from, to))
    })?;
    let span = to.at_ms.saturating_sub(from.at_ms).max(1);
    let local = t_ms.saturating_sub(from.at_ms);
    let ratio = ease(local as f32 / span as f32, easing);
    interpolate(from.value, to.value, ratio)
}

fn normalize_elapsed(elapsed_ms: u32, duration: u32, loop_mode: LoopMode) -> u32 {
    match loop_mode {
        LoopMode::Once => elapsed_ms.min(duration),
        LoopMode::Loop => elapsed_ms % duration,
        LoopMode::PingPong => {
            let cycle = duration.saturating_mul(2).max(1);
            let t = elapsed_ms % cycle;
            if t > duration {
                cycle.saturating_sub(t)
            } else {
                t
            }
        }
        LoopMode::RepeatN(count) => elapsed_ms.min(duration.saturating_mul(u32::from(count))),
    }
}

fn ease(t: f32, easing: Easing) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match easing {
        Easing::Linear => t,
        Easing::EaseIn => t * t,
        Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
        Easing::EaseInOut => {
            if t < 0.5 {
                2.0 * t * t
            } else {
                1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
            }
        }
        Easing::Bounce | Easing::Spring => t,
    }
}

fn interpolate(from: AnimatableValue, to: AnimatableValue, ratio: f32) -> Option<AnimatableValue> {
    match (from, to) {
        (AnimatableValue::I32(from), AnimatableValue::I32(to)) => {
            Some(AnimatableValue::I32(lerp_i32(from, to, ratio)))
        }
        (AnimatableValue::U8(from), AnimatableValue::U8(to)) => Some(AnimatableValue::U8(
            lerp_i32(i32::from(from), i32::from(to), ratio) as u8,
        )),
        (AnimatableValue::Rgb(from), AnimatableValue::Rgb(to)) => {
            let r = lerp_i32(
                ((from >> 16) & 0xff) as i32,
                ((to >> 16) & 0xff) as i32,
                ratio,
            );
            let g = lerp_i32(
                ((from >> 8) & 0xff) as i32,
                ((to >> 8) & 0xff) as i32,
                ratio,
            );
            let b = lerp_i32((from & 0xff) as i32, (to & 0xff) as i32, ratio);
            Some(AnimatableValue::Rgb(
                ((r as u32) << 16) | ((g as u32) << 8) | b as u32,
            ))
        }
        _ => None,
    }
}

fn lerp_i32(from: i32, to: i32, ratio: f32) -> i32 {
    (from as f32 + (to - from) as f32 * ratio).round() as i32
}

fn mix_signature(current: u64, value: u64) -> u64 {
    current.wrapping_mul(0x100_0000_01b3).wrapping_add(value)
}

fn property_signature(property: AnimatableProp) -> u64 {
    match property {
        AnimatableProp::Opacity => 1,
        AnimatableProp::X => 2,
        AnimatableProp::Y => 3,
        AnimatableProp::Width => 4,
        AnimatableProp::Height => 5,
        AnimatableProp::Scale => 6,
        AnimatableProp::AccentMix => 7,
        AnimatableProp::BorderWidth => 8,
        AnimatableProp::ShadowRadius => 9,
        AnimatableProp::SelectionOffset => 10,
        AnimatableProp::ProgressPermille => 11,
    }
}

fn value_signature(value: AnimatableValue) -> u64 {
    match value {
        AnimatableValue::I32(value) => (value / 32) as u64,
        AnimatableValue::U8(value) => u64::from(value / 32),
        AnimatableValue::Rgb(value) => {
            let r = (value >> 16) & 0xff;
            let g = (value >> 8) & 0xff;
            let b = value & 0xff;
            ((r / 32) << 8 | (g / 32) << 4 | (b / 32)) as u64
        }
    }
}
