pub mod assets;
pub mod framebuffer;
pub mod lvgl;
pub mod null;

use anyhow::Result;
use yoyopod_protocol::ui::UiScreen;

use crate::presentation::registry::DirtyRegion;
use crate::presentation::screens::ScreenModel;
use crate::presentation::transitions::TransitionSampler;

pub use framebuffer::Framebuffer;
pub use lvgl::LvglRenderer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub renderer: &'static str,
    pub screen: UiScreen,
    pub dirty_region: Option<DirtyRegion>,
}

pub trait Renderer {
    fn render(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
        dirty_region: Option<DirtyRegion>,
    ) -> Result<RenderReport>;
}
