use anyhow::Result;

use crate::animation::TransitionSampler;
use crate::engine::DirtyRegion;
use crate::presentation::view_models::ScreenModel;
use crate::renderer::{Framebuffer, RenderReport, Renderer};

#[derive(Debug, Default)]
pub struct NullRenderer {
    reports: Vec<RenderReport>,
}

impl NullRenderer {
    pub fn reports(&self) -> &[RenderReport] {
        &self.reports
    }
}

impl Renderer for NullRenderer {
    fn render(
        &mut self,
        _framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        _transitions: &TransitionSampler<'_>,
        dirty_region: Option<DirtyRegion>,
    ) -> Result<RenderReport> {
        let report = RenderReport {
            renderer: "null",
            screen: model.screen(),
            dirty_region,
        };
        self.reports.push(report.clone());
        Ok(report)
    }
}
