#[cfg(feature = "native-lvgl")]
use std::path::Path;

use anyhow::{bail, Result};

use crate::animation::TransitionSampler;
use crate::presentation::view_models::{ScreenModel, StatusBarModel};
use crate::router::{self, NativeRenderScene};
use yoyopod_protocol::ui::UiScreen;

use super::list_view::rust_owned_scene_model;
use super::scene_controller::{controller_for_native_scene, NativeSceneController};
use crate::renderer::widgets::LvglFacade;

const fn native_scene_for_screen(screen: UiScreen) -> NativeRenderScene {
    router::screen_entry(screen).native_scene
}

pub trait SceneBridge {
    fn build_scene(&mut self, scene: NativeRenderScene) -> Result<()>;
    fn sync_status(&mut self, status: &StatusBarModel) -> Result<()>;
    fn sync_scene(
        &mut self,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()>;
    fn destroy_scene(&mut self, scene: NativeRenderScene);
    fn clear_screen(&mut self) -> Result<()>;
}

pub struct NativeSceneRenderer<B> {
    bridge: B,
    active_scene: Option<NativeRenderScene>,
    active_screen: Option<UiScreen>,
}

impl<B> NativeSceneRenderer<B>
where
    B: SceneBridge,
{
    pub fn new(bridge: B) -> Self {
        Self {
            bridge,
            active_scene: None,
            active_screen: None,
        }
    }

    pub fn render(
        &mut self,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        let screen = model.screen();
        let scene = native_scene_for_screen(screen);

        if self.active_scene != Some(scene) {
            if let Some(active_scene) = self.active_scene.take() {
                self.bridge.destroy_scene(active_scene);
            }
            self.bridge.build_scene(scene)?;
            self.active_scene = Some(scene);
        }

        self.bridge.sync_status(&model.chrome().status)?;
        self.bridge.sync_scene(model, transitions)?;
        self.active_screen = Some(screen);
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        if let Some(active_scene) = self.active_scene.take() {
            self.bridge.destroy_scene(active_scene);
        }
        self.bridge.clear_screen()?;
        self.active_screen = None;
        Ok(())
    }

    pub fn active_scene(&self) -> Option<NativeRenderScene> {
        self.active_scene
    }

    pub fn active_screen(&self) -> Option<UiScreen> {
        self.active_screen
    }

    pub fn bridge(&self) -> &B {
        &self.bridge
    }

    pub fn bridge_mut(&mut self) -> &mut B {
        &mut self.bridge
    }
}

pub struct RustSceneBridge<F> {
    facade: F,
    controller: Option<NativeSceneController>,
    active_scene: Option<NativeRenderScene>,
    last_status: Option<StatusBarModel>,
}

impl<F> RustSceneBridge<F>
where
    F: LvglFacade,
{
    pub fn new(facade: F) -> Self {
        Self {
            facade,
            controller: None,
            active_scene: None,
            last_status: None,
        }
    }

    pub fn active_scene(&self) -> Option<NativeRenderScene> {
        self.active_scene
    }

    pub fn last_status(&self) -> Option<&StatusBarModel> {
        self.last_status.as_ref()
    }

    pub fn facade(&self) -> &F {
        &self.facade
    }

    pub fn facade_mut(&mut self) -> &mut F {
        &mut self.facade
    }
}

impl<F> SceneBridge for RustSceneBridge<F>
where
    F: LvglFacade,
{
    fn build_scene(&mut self, scene: NativeRenderScene) -> Result<()> {
        if self.active_scene == Some(scene) {
            return Ok(());
        }
        if self.active_scene.is_some() {
            if let Some(controller) = self.controller.as_mut() {
                let _ = controller.teardown(&mut self.facade);
            }
        }
        self.controller = Some(controller_for_native_scene(scene)?);
        self.active_scene = Some(scene);
        Ok(())
    }

    fn sync_status(&mut self, status: &StatusBarModel) -> Result<()> {
        self.last_status = Some(status.clone());
        Ok(())
    }

    fn sync_scene(
        &mut self,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        let expected_scene = native_scene_for_screen(model.screen());
        let Some(active_scene) = self.active_scene else {
            bail!(
                "{} scene must be built before sync",
                expected_scene.as_str()
            );
        };
        if active_scene != expected_scene {
            bail!(
                "Rust LVGL scene bridge built {} but received {} model",
                active_scene.as_str(),
                model.screen().as_str()
            );
        }
        let model = rust_owned_scene_model(model, active_scene);
        let controller = self
            .controller
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Rust LVGL scene bridge has no active controller"))?;
        controller.sync(&mut self.facade, &model, transitions)
    }

    fn destroy_scene(&mut self, scene: NativeRenderScene) {
        if self.active_scene == Some(scene) {
            if let Some(controller) = self.controller.as_mut() {
                let _ = controller.teardown(&mut self.facade);
            }
            self.controller = None;
            self.active_scene = None;
        }
    }

    fn clear_screen(&mut self) -> Result<()> {
        if let Some(controller) = self.controller.as_mut() {
            controller.teardown(&mut self.facade)?;
        }
        self.controller = None;
        self.active_scene = None;
        Ok(())
    }
}
#[cfg(feature = "native-lvgl")]
impl RustSceneBridge<crate::renderer::lvgl::NativeLvglFacade> {
    pub fn open(explicit_source: Option<&Path>) -> Result<Self> {
        Ok(Self::new(crate::renderer::lvgl::NativeLvglFacade::open(
            explicit_source,
        )?))
    }

    pub fn display_needs_reset(&self, framebuffer: &crate::renderer::Framebuffer) -> bool {
        self.facade().display_needs_reset(framebuffer)
    }

    pub fn ensure_display_registered(
        &mut self,
        framebuffer: &crate::renderer::Framebuffer,
    ) -> Result<()> {
        self.facade_mut().ensure_display_registered(framebuffer)
    }

    pub fn render_frame(&mut self, framebuffer: &mut crate::renderer::Framebuffer) -> Result<()> {
        self.facade_mut().render_frame(framebuffer)
    }
}
