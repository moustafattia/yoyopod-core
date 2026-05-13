#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRegion {
    pub x: u16,
    pub y: u16,
    pub w: u16,
    pub h: u16,
}

impl DirtyRegion {
    pub const fn union(self, other: Self) -> Self {
        let x0 = if self.x < other.x { self.x } else { other.x };
        let y0 = if self.y < other.y { self.y } else { other.y };
        let self_x1 = self.x.saturating_add(self.w);
        let self_y1 = self.y.saturating_add(self.h);
        let other_x1 = other.x.saturating_add(other.w);
        let other_y1 = other.y.saturating_add(other.h);
        let x1 = if self_x1 > other_x1 {
            self_x1
        } else {
            other_x1
        };
        let y1 = if self_y1 > other_y1 {
            self_y1
        } else {
            other_y1
        };
        Self {
            x: x0,
            y: y0,
            w: x1.saturating_sub(x0),
            h: y1.saturating_sub(y0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    FullFrame,
    HudRegion,
    Region(DirtyRegion),
}

pub fn render_mode_for_region(region: Option<DirtyRegion>, hud_region: DirtyRegion) -> RenderMode {
    match region {
        Some(region) if region == hud_region => RenderMode::HudRegion,
        Some(region) => RenderMode::Region(region),
        None => RenderMode::FullFrame,
    }
}
