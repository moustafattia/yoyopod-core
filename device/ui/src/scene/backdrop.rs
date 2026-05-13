#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backdrop {
    Solid(u32),
    Gradient { from: u32, to: u32, angle_deg: i16 },
    AccentDrift { accent: u32, speed_ms: u32 },
    Vignette { base: u32, falloff: u8 },
}
