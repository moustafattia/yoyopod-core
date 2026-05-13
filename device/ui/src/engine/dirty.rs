use crate::render_contract::{DirtyRegion, RenderMode};

pub fn render_mode_for_region(region: Option<DirtyRegion>, hud_region: DirtyRegion) -> RenderMode {
    match region {
        Some(region) if region == hud_region => RenderMode::HudRegion,
        Some(region) => RenderMode::Region(region),
        None => RenderMode::FullFrame,
    }
}
