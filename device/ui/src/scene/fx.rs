use crate::animation::ActorRef;
use crate::components::widgets::{
    progress_sweep, voice_meter, ProgressSweepProps, VoiceMeterProps,
};
use crate::engine::{Element, Key};
use crate::scene::roles;
use crate::ElementKind;

use super::RegionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FxLayerId(pub u8);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FxLayer {
    pub halos: Vec<Halo>,
    pub pulses: Vec<PulseRing>,
    pub particles: Vec<ParticleField>,
    pub glows: Vec<GlowBloom>,
    pub progress_sweeps: Vec<ProgressSweep>,
    pub voice_meters: Vec<VoiceMeter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Halo {
    pub target: ActorRef,
    pub color: u32,
    pub period_ms: u32,
    pub min_opacity: u8,
    pub max_opacity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PulseRing {
    pub target: ActorRef,
    pub color: u32,
    pub duration_ms: u32,
    pub max_radius: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticleField {
    pub region: RegionId,
    pub count: u8,
    pub color: u32,
    pub drift_speed_ms: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlowBloom {
    pub target: ActorRef,
    pub blur: u8,
    pub intensity: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressSweep {
    pub progress_permille: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceMeter {
    pub level_permille: i32,
    pub recording: bool,
}

impl FxLayer {
    pub fn element(&self) -> Option<Element> {
        if self.halos.is_empty()
            && self.pulses.is_empty()
            && self.particles.is_empty()
            && self.glows.is_empty()
            && self.progress_sweeps.is_empty()
            && self.voice_meters.is_empty()
        {
            return None;
        }

        let mut element = Element::new(ElementKind::Container, Some(roles::SCENE_FX))
            .key(Key::Static("scene_fx"));
        for (index, halo) in self.halos.iter().enumerate() {
            element = element.child(halo_element(index, halo));
        }
        for (index, pulse) in self.pulses.iter().enumerate() {
            element = element.child(pulse_element(index, pulse));
        }
        for (field_index, field) in self.particles.iter().enumerate() {
            for index in 0..field.count.min(8) {
                element = element.child(particle_element(field_index, index, field));
            }
        }
        for (index, glow) in self.glows.iter().enumerate() {
            element = element.child(glow_element(index, glow));
        }
        for (index, sweep) in self.progress_sweeps.iter().enumerate() {
            element = element.child(progress_sweep_element(index, sweep));
        }
        for (index, meter) in self.voice_meters.iter().enumerate() {
            element = element.child(voice_meter_element(index, meter));
        }
        Some(element)
    }
}

fn halo_element(index: usize, halo: &Halo) -> Element {
    fx_target_element(
        roles::FX_HALO,
        Key::String(format!("fx:halo:{index}")),
        halo.target,
    )
    .accent(halo.color)
    .with_opacity(halo.max_opacity)
}

fn pulse_element(index: usize, pulse: &PulseRing) -> Element {
    fx_target_element(
        roles::FX_PULSE,
        Key::String(format!("fx:pulse:{index}")),
        pulse.target,
    )
    .accent(pulse.color)
    .with_opacity(96)
}

fn particle_element(field_index: usize, index: u8, field: &ParticleField) -> Element {
    Element::new(ElementKind::Container, Some(roles::FX_PARTICLE))
        .key(Key::String(format!("fx:particle:{field_index}:{index}")))
        .region(field.region)
        .accent(field.color)
}

fn glow_element(index: usize, glow: &GlowBloom) -> Element {
    let role = match glow.target {
        ActorRef::Screen => roles::FX_SPINNER,
        _ => roles::FX_GLOW,
    };
    fx_target_element(role, Key::String(format!("fx:glow:{index}")), glow.target)
        .with_opacity(glow.intensity)
}

fn progress_sweep_element(index: usize, sweep: &ProgressSweep) -> Element {
    progress_sweep(ProgressSweepProps {
        progress_permille: sweep.progress_permille,
    })
    .key(Key::String(format!("fx:progress_sweep:{index}")))
}

fn voice_meter_element(index: usize, meter: &VoiceMeter) -> Element {
    voice_meter(VoiceMeterProps {
        level_permille: meter.level_permille,
        recording: meter.recording,
    })
    .key(Key::String(format!("fx:voice_meter:{index}")))
}

fn fx_target_element(role: &'static str, key: Key, target: ActorRef) -> Element {
    let element = Element::new(ElementKind::Container, Some(role))
        .key(key)
        .actor(target);
    match target {
        ActorRef::Region(region) => element.region(region),
        ActorRef::Screen
        | ActorRef::DeckItem { .. }
        | ActorRef::FxNode { .. }
        | ActorRef::Cursor => element,
    }
}

trait FxElementExt {
    fn with_opacity(self, opacity: u8) -> Self;
}

impl FxElementExt for Element {
    fn with_opacity(mut self, opacity: u8) -> Self {
        self.props.opacity = Some(opacity);
        self
    }
}
