use crate::renderer::assets::RenderAssets;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayoutRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

pub struct LayoutResolver<'a> {
    assets: &'a RenderAssets,
}

impl<'a> LayoutResolver<'a> {
    pub const fn new(assets: &'a RenderAssets) -> Self {
        Self { assets }
    }

    pub fn resolve_role(&self, role: &str, occurrence: usize) -> Option<LayoutRect> {
        self.assets.layout_role(role).map(|layout| LayoutRect {
            x: layout.x + layout.repeat_x.unwrap_or(0) * occurrence as i32,
            y: layout.y + layout.repeat_y.unwrap_or(0) * occurrence as i32,
            width: layout.width,
            height: layout.height,
        })
    }
}
