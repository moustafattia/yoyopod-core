pub mod assets;
pub mod framebuffer;
#[cfg(feature = "native-lvgl")]
pub mod lvgl;
pub mod null;
pub mod pipeline;
pub mod screens;
pub mod styling;
pub mod widgets;

use anyhow::Result;
use yoyopod_protocol::ui::UiScreen;

use crate::animation::TransitionSampler;
use crate::engine::DirtyRegion;
use crate::presentation::view_models::ScreenModel;

pub use framebuffer::Framebuffer;
pub use pipeline::LvglRenderer;

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
