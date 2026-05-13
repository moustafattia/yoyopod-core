pub mod assets;
pub mod framebuffer;
#[cfg(feature = "native-lvgl")]
pub mod lvgl;
mod lvgl_controllers;
pub mod lvgl_renderer;
pub mod node_registry;
pub mod null_renderer;
pub mod styling;
pub mod widgets;

use anyhow::Result;
use yoyopod_protocol::ui::UiScreen;

use crate::animation::TransitionSampler;
use crate::engine::{DirtyRegion, Mutation};
use crate::presentation::view_models::ScreenModel;
use crate::renderer::framebuffer::Framebuffer as RenderFramebuffer;

pub use framebuffer::Framebuffer;
pub use lvgl_renderer::LvglRenderer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub renderer: &'static str,
    pub mode: RenderMode,
    pub widget_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderMode {
    FullFrame,
    HudRegion,
    Region(DirtyRegion),
}

pub trait Renderer {
    fn apply(&mut self, mutations: &[Mutation]) -> Result<()>;

    fn flush(
        &mut self,
        framebuffer: &mut RenderFramebuffer,
        mode: RenderMode,
    ) -> Result<RenderReport>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenRenderReport {
    pub renderer: &'static str,
    pub screen: UiScreen,
    pub dirty_region: Option<DirtyRegion>,
}

pub trait ScreenRenderer {
    fn render(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
        dirty_region: Option<DirtyRegion>,
    ) -> Result<ScreenRenderReport>;
}
