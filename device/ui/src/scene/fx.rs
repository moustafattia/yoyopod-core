use crate::animation::{presets, ActorRef, TimelineRef, TrackIndex};
use crate::engine::{AnimSlot, Element, Key};
use crate::render_contract::ElementKind;

use super::RegionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FxLayerId(pub u8);

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FxLayer {
    pub halos: Vec<Halo>,
    pub pulses: Vec<PulseRing>,
    pub particles: Vec<ParticleField>,
    pub glows: Vec<GlowBloom>,
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

impl FxLayer {
    pub fn element(&self) -> Option<Element> {
        if self.halos.is_empty()
            && self.pulses.is_empty()
            && self.particles.is_empty()
            && self.glows.is_empty()
        {
            return None;
        }

        let mut element =
            Element::new(ElementKind::Container, Some("scene_fx")).key(Key::Static("scene_fx"));
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
        Some(element)
    }
}

fn halo_element(index: usize, halo: &Halo) -> Element {
    Element::new(ElementKind::Container, Some("fx_halo"))
        .key(Key::String(format!("fx:halo:{index}")))
        .accent(halo.color)
        .with_anim(AnimSlot {
            timeline: TimelineRef(presets::BREATHE_TIMELINE_ID),
            track: TrackIndex(0),
        })
        .with_opacity(halo.max_opacity)
}

fn pulse_element(index: usize, pulse: &PulseRing) -> Element {
    Element::new(ElementKind::Container, Some("fx_pulse"))
        .key(Key::String(format!("fx:pulse:{index}")))
        .accent(pulse.color)
        .with_opacity(96)
}

fn particle_element(field_index: usize, index: u8, field: &ParticleField) -> Element {
    Element::new(ElementKind::Container, Some("fx_particle"))
        .key(Key::String(format!("fx:particle:{field_index}:{index}")))
        .accent(field.color)
}

fn glow_element(index: usize, glow: &GlowBloom) -> Element {
    let role = match glow.target {
        ActorRef::Screen => "fx_spinner",
        _ => "fx_glow",
    };
    Element::new(ElementKind::Container, Some(role))
        .key(Key::String(format!("fx:glow:{index}")))
        .with_opacity(glow.intensity)
}

trait FxElementExt {
    fn with_anim(self, anim: AnimSlot) -> Self;
    fn with_opacity(self, opacity: u8) -> Self;
}

impl FxElementExt for Element {
    fn with_anim(mut self, anim: AnimSlot) -> Self {
        self.anim = Some(anim);
        self
    }

    fn with_opacity(mut self, opacity: u8) -> Self {
        self.props.opacity = Some(opacity);
        self
    }
}
