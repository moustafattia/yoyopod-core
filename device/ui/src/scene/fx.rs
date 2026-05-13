use crate::animation::ActorRef;

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
