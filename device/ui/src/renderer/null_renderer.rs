use anyhow::Result;

use crate::render_contract::{Mutation, RenderMode};
use crate::renderer::{Framebuffer, RenderReport, Renderer};

#[derive(Debug, Default)]
pub struct NullRenderer {
    mutations: Vec<Mutation>,
}

impl NullRenderer {
    pub fn mutations(&self) -> &[Mutation] {
        &self.mutations
    }

    pub fn mutation_count(&self) -> usize {
        self.mutations.len()
    }

    pub fn clear(&mut self) {
        self.mutations.clear();
    }
}

impl Renderer for NullRenderer {
    fn apply(&mut self, mutations: &[Mutation]) -> Result<()> {
        self.mutations.extend_from_slice(mutations);
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
