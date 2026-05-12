use crate::lvgl::theme::{self as legacy_theme, WidgetStyle};
use crate::render::assets::RenderAssets;

pub struct ThemeResolver<'a> {
    assets: &'a RenderAssets,
}

impl<'a> ThemeResolver<'a> {
    pub const fn new(assets: &'a RenderAssets) -> Self {
        Self { assets }
    }

    pub fn style_for_role(&self, role: &str) -> WidgetStyle {
        self.assets
            .theme_role(role)
            .map(|theme_role| {
                let mut style = legacy_theme::style_for_role(role);
                if let Some(fill_rgb) = theme_role.fill_rgb {
                    style.bg_color = Some(fill_rgb);
                }
                if let Some(text_rgb) = theme_role.text_rgb {
                    style.text_color = Some(text_rgb);
                }
                if let Some(opacity) = theme_role.opacity {
                    style.bg_opa = opacity;
                }
                style
            })
            .unwrap_or_else(|| legacy_theme::style_for_role(role))
    }

    pub fn style_for_selected_role(&self, role: &str, selected: bool) -> WidgetStyle {
        if selected {
            legacy_theme::style_for_selected_role(role, true)
        } else {
            self.style_for_role(role)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::assets;
    use crate::render::lvgl::roles;

    #[test]
    fn resolves_asset_backed_role_style() {
        let render_assets = assets::load_render_assets().unwrap();
        let resolver = ThemeResolver::new(&render_assets);

        let style = resolver.style_for_role(roles::OVERLAY_SUBTITLE);

        assert_eq!(style.text_color, Some(legacy_theme::MUTED_RGB));
    }

    #[test]
    fn falls_back_to_legacy_role_style() {
        let render_assets = assets::load_render_assets().unwrap();
        let resolver = ThemeResolver::new(&render_assets);

        assert_eq!(
            resolver.style_for_role("hub_title"),
            legacy_theme::style_for_role("hub_title")
        );
    }
}
