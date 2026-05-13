use anyhow::Result;

use crate::render_contract::{Mutation, RenderMode};
use crate::renderer::{Framebuffer, RenderReport, Renderer};

#[derive(Debug, Default)]
pub struct NullRenderer {
    mutation_count: usize,
}

impl NullRenderer {
    pub fn mutation_count(&self) -> usize {
        self.mutation_count
    }
}

impl Renderer for NullRenderer {
    fn apply(&mut self, mutations: &[Mutation]) -> Result<()> {
        self.mutation_count += mutations.len();
        Ok(())
    }

    fn flush(&mut self, _framebuffer: &mut Framebuffer, mode: RenderMode) -> Result<RenderReport> {
        Ok(RenderReport {
            renderer: "null",
            mode,
            widget_count: 0,
        })
    }
}
