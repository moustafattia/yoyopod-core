use std::path::Path;

use anyhow::Result;

#[cfg(feature = "native-lvgl")]
pub mod backend;
pub mod controllers;
pub mod facade;
#[cfg(feature = "native-lvgl")]
pub(crate) mod ffi;
#[cfg(feature = "native-lvgl")]
pub(crate) mod icons;
pub mod layout;
pub mod primitives;
pub(crate) mod roles;
pub mod scene;
pub mod style;
pub mod theme;

use crate::framebuffer::Framebuffer;
use crate::screens::ScreenModel;
#[cfg(feature = "native-lvgl")]
use backend::NativeLvglFacade;
#[cfg(feature = "native-lvgl")]
use scene::{NativeSceneRenderer, RustSceneBridge, SceneBridge};

pub use facade::LvglFacade;
pub use primitives::WidgetId;
pub use scene::NativeSceneKey;

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
    ) -> Result<()> {
        self.renderer.render_screen_model(framebuffer, model)
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
    ) -> Result<()> {
        if self.renderer.bridge().display_needs_reset(framebuffer) {
            self.renderer.clear()?;
        }
        self.renderer
            .bridge_mut()
            .ensure_display_registered(framebuffer)?;
        self.renderer.render(model)?;
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
    ) -> Result<()> {
        anyhow::bail!("native-lvgl feature is disabled for this build")
    }
}
