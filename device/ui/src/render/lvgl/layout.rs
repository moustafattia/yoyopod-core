use crate::render::assets::RenderAssets;

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

    pub fn resolve_role(&self, role: &str) -> Option<LayoutRect> {
        self.assets.layout_role(role).map(|layout| LayoutRect {
            x: layout.x,
            y: layout.y,
            width: layout.width,
            height: layout.height,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::assets;
    use crate::render::lvgl::roles;

    #[test]
    fn resolves_asset_backed_role_layout() {
        let render_assets = assets::load_render_assets().unwrap();
        let resolver = LayoutResolver::new(&render_assets);

        assert_eq!(
            resolver.resolve_role(roles::OVERLAY_TITLE),
            Some(LayoutRect {
                x: 18,
                y: 96,
                width: 204,
                height: 34,
            })
        );
    }

    #[test]
    fn unknown_role_returns_none_for_legacy_fallback() {
        let render_assets = assets::load_render_assets().unwrap();
        let resolver = LayoutResolver::new(&render_assets);

        assert_eq!(resolver.resolve_role("legacy_only_role"), None);
    }
}
