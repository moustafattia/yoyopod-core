use crate::runtime::UiScreen;

pub use crate::lvgl_bridge::LvglRenderer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererMode {
    Auto,
    Lvgl,
    Framebuffer,
}

impl RendererMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Lvgl => "lvgl",
            Self::Framebuffer => "framebuffer",
        }
    }
}

#[derive(Debug, Default)]
pub struct RendererState {
    active_screen: Option<UiScreen>,
}

impl RendererState {
    pub fn needs_rebuild(&self, screen: UiScreen) -> bool {
        self.active_screen != Some(screen)
    }

    pub fn mark_screen_built(&mut self, screen: UiScreen) {
        self.active_screen = Some(screen);
    }

    pub fn clear(&mut self) {
        self.active_screen = None;
    }

    pub fn active_screen(&self) -> Option<UiScreen> {
        self.active_screen
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renderer_mode_names_are_stable() {
        assert_eq!(RendererMode::Auto.as_str(), "auto");
        assert_eq!(RendererMode::Lvgl.as_str(), "lvgl");
        assert_eq!(RendererMode::Framebuffer.as_str(), "framebuffer");
    }

    #[test]
    fn persistent_renderer_tracks_last_screen() {
        let mut state = RendererState::default();

        assert!(state.needs_rebuild(UiScreen::Hub));
        state.mark_screen_built(UiScreen::Hub);
        assert!(!state.needs_rebuild(UiScreen::Hub));
        assert!(state.needs_rebuild(UiScreen::Listen));
    }
}
