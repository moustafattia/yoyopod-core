pub mod assets;
pub mod framebuffer;
pub mod lvgl;
pub mod node_registry;
pub mod null_renderer;
pub mod styling;
pub mod widgets;

use anyhow::Result;

use crate::renderer::framebuffer::Framebuffer as RenderFramebuffer;
use crate::{Mutation, RenderMode};

pub use framebuffer::Framebuffer;
pub use lvgl::LvglRenderer;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderReport {
    pub renderer: &'static str,
    pub mode: RenderMode,
    pub widget_count: usize,
}

pub trait Renderer {
    fn apply(&mut self, mutations: &[Mutation]) -> Result<()>;

    fn flush(
        &mut self,
        framebuffer: &mut RenderFramebuffer,
        mode: RenderMode,
    ) -> Result<RenderReport>;
}
