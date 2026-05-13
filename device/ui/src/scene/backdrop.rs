use crate::engine::{Element, Key};
use crate::render_contract::ElementKind;
use crate::scene::roles;
use crate::scene::RegionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backdrop {
    Solid(u32),
    Gradient { from: u32, to: u32, angle_deg: i16 },
    AccentDrift { accent: u32, speed_ms: u32 },
    Vignette { base: u32, falloff: u8 },
}

impl Backdrop {
    pub fn element(self) -> Element {
        let mut element = Element::new(ElementKind::Container, Some(roles::SCENE_BACKDROP))
            .key(Key::Static("backdrop"))
            .region(RegionId::Backdrop);
        element.props.variant = Some(match self {
            Self::Solid(_) => "solid",
            Self::Gradient { .. } => "gradient",
            Self::AccentDrift { .. } => "accent_drift",
            Self::Vignette { .. } => "vignette",
        });
        element.props.accent = Some(match self {
            Self::Solid(rgb) => rgb,
            Self::Gradient { from, .. } => from,
            Self::AccentDrift { accent, .. } => accent,
            Self::Vignette { base, .. } => base,
        });
        element
    }
}
