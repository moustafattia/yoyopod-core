use anyhow::Result;

use crate::animation::TransitionSampler;
use crate::engine::{DirtyRegion, Mutation};
use crate::presentation::view_models::ScreenModel;
use crate::renderer::{
    Framebuffer, RenderMode, RenderReport, Renderer, ScreenRenderReport, ScreenRenderer,
};

#[derive(Debug, Default)]
pub struct NullRenderer {
    reports: Vec<ScreenRenderReport>,
    mutation_count: usize,
}

impl NullRenderer {
    pub fn reports(&self) -> &[ScreenRenderReport] {
        &self.reports
    }

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

impl ScreenRenderer for NullRenderer {
    fn render(
        &mut self,
        _framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        _transitions: &TransitionSampler<'_>,
        dirty_region: Option<DirtyRegion>,
    ) -> Result<ScreenRenderReport> {
        let report = ScreenRenderReport {
            renderer: "null",
            screen: model.screen(),
            dirty_region,
        };
        self.reports.push(report.clone());
        Ok(report)
    }
}
