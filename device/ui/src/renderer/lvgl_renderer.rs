use std::path::Path;

use anyhow::Result;

#[path = "pipeline/list_view.rs"]
mod list_view;
#[path = "pipeline/scene.rs"]
pub mod scene;
#[path = "pipeline/scene_controller.rs"]
mod scene_controller;

use crate::animation::TransitionSampler;
use crate::presentation::view_models::ScreenModel;
#[cfg(feature = "native-lvgl")]
use crate::renderer::lvgl::NativeLvglFacade;
use crate::renderer::{Framebuffer, RenderReport, Renderer};
#[cfg(feature = "native-lvgl")]
use scene::{NativeSceneRenderer, RustSceneBridge, SceneBridge};

#[cfg(not(feature = "native-lvgl"))]
pub struct LvglRenderer;

#[cfg(feature = "native-lvgl")]
pub struct LvglRenderer {
    renderer: RuntimeSceneLvglRenderer<RustSceneBridge<NativeLvglFacade>>,
}

#[cfg(feature = "native-lvgl")]
impl LvglRenderer {
    pub fn open(explicit_source: Option<&Path>) -> Result<Self> {
        let renderer = RuntimeSceneLvglRenderer::new(RustSceneBridge::open(explicit_source)?);
        Ok(Self { renderer })
    }

    pub fn render_screen_model(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        self.renderer
            .render_screen_model(framebuffer, model, transitions)
    }
}

#[cfg(feature = "native-lvgl")]
impl Renderer for LvglRenderer {
    fn render(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
        dirty_region: Option<crate::engine::DirtyRegion>,
    ) -> Result<RenderReport> {
        self.render_screen_model(framebuffer, model, transitions)?;
        Ok(RenderReport {
            renderer: "lvgl",
            screen: model.screen(),
            dirty_region,
        })
    }
}

#[cfg(feature = "native-lvgl")]
trait RuntimeSceneBridge: SceneBridge {
    fn display_needs_reset(&self, framebuffer: &Framebuffer) -> bool;
    fn ensure_display_registered(&mut self, framebuffer: &Framebuffer) -> Result<()>;
    fn render_frame(&mut self, framebuffer: &mut Framebuffer) -> Result<()>;
}

#[cfg(feature = "native-lvgl")]
impl RuntimeSceneBridge for RustSceneBridge<NativeLvglFacade> {
    fn display_needs_reset(&self, framebuffer: &Framebuffer) -> bool {
        RustSceneBridge::<NativeLvglFacade>::display_needs_reset(self, framebuffer)
    }

    fn ensure_display_registered(&mut self, framebuffer: &Framebuffer) -> Result<()> {
        RustSceneBridge::<NativeLvglFacade>::ensure_display_registered(self, framebuffer)
    }

    fn render_frame(&mut self, framebuffer: &mut Framebuffer) -> Result<()> {
        RustSceneBridge::<NativeLvglFacade>::render_frame(self, framebuffer)
    }
}

#[cfg(feature = "native-lvgl")]
struct RuntimeSceneLvglRenderer<B> {
    renderer: NativeSceneRenderer<B>,
}

#[cfg(feature = "native-lvgl")]
impl<B> RuntimeSceneLvglRenderer<B>
where
    B: RuntimeSceneBridge,
{
    fn new(bridge: B) -> Self {
        Self {
            renderer: NativeSceneRenderer::new(bridge),
        }
    }

    fn render_screen_model(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        if self.renderer.bridge().display_needs_reset(framebuffer) {
            self.renderer.clear()?;
        }
        self.renderer
            .bridge_mut()
            .ensure_display_registered(framebuffer)?;
        self.renderer.render(model, transitions)?;
        self.renderer.bridge_mut().render_frame(framebuffer)
    }
}

#[cfg(not(feature = "native-lvgl"))]
impl LvglRenderer {
    pub fn open(_explicit_source: Option<&Path>) -> Result<Self> {
        anyhow::bail!("native-lvgl feature is disabled for this build")
    }

    pub fn render_screen_model(
        &mut self,
        _framebuffer: &mut Framebuffer,
        _model: &ScreenModel,
        _transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        anyhow::bail!("native-lvgl feature is disabled for this build")
    }
}

#[cfg(not(feature = "native-lvgl"))]
impl Renderer for LvglRenderer {
    fn render(
        &mut self,
        _framebuffer: &mut Framebuffer,
        _model: &ScreenModel,
        _transitions: &TransitionSampler<'_>,
        _dirty_region: Option<crate::engine::DirtyRegion>,
    ) -> Result<RenderReport> {
        anyhow::bail!("native-lvgl feature is disabled for this build")
    }
}
