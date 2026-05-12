pub mod controllers;
pub mod primitives;
pub(crate) mod roles;
pub mod scene_backend;
pub mod theme;

use std::path::Path;

#[cfg(not(feature = "native-lvgl"))]
use anyhow::bail;
use anyhow::{anyhow, Result};

use crate::presentation::registry::{
    self, ControllerKind, NativeRenderScene, RenderScene, ScreenRegistryEntry,
};
use crate::runtime::UiScreen;
use crate::screens::ScreenModel;

#[cfg(feature = "native-lvgl")]
pub use crate::render::lvgl::backend::NativeLvglFacade;
pub use controllers::{
    typed_controller, AskController, CallController, HubController, ListController,
    ListenController, NowPlayingController, OverlayController, PlaylistController, PowerController,
    ScreenController, TalkActionsController, TalkController, TypedScreenController,
};
pub use primitives::WidgetId;
pub use scene_backend::{NativeSceneRenderer, RustSceneBridge, SceneBridge};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneKey {
    Hub,
    List,
    NowPlaying,
    Ask,
    TalkActions,
    Call,
    Power,
    Overlay,
}

impl SceneKey {
    pub const fn for_screen(screen: UiScreen) -> Self {
        match registry::screen_entry(screen).render_scene {
            RenderScene::Hub => Self::Hub,
            RenderScene::List => Self::List,
            RenderScene::NowPlaying => Self::NowPlaying,
            RenderScene::Ask => Self::Ask,
            RenderScene::TalkActions => Self::TalkActions,
            RenderScene::Call => Self::Call,
            RenderScene::Power => Self::Power,
            RenderScene::Overlay => Self::Overlay,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hub => "hub",
            Self::List => "list",
            Self::NowPlaying => "now_playing",
            Self::Ask => "ask",
            Self::TalkActions => "talk_actions",
            Self::Call => "call",
            Self::Power => "power",
            Self::Overlay => "overlay",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSceneKey {
    Hub,
    Listen,
    Playlist,
    NowPlaying,
    Talk,
    TalkActions,
    IncomingCall,
    OutgoingCall,
    InCall,
    Ask,
    Power,
    Overlay,
}

impl NativeSceneKey {
    pub const fn for_screen(screen: UiScreen) -> Self {
        match registry::screen_entry(screen).native_scene {
            NativeRenderScene::Hub => Self::Hub,
            NativeRenderScene::Listen => Self::Listen,
            NativeRenderScene::Playlist => Self::Playlist,
            NativeRenderScene::NowPlaying => Self::NowPlaying,
            NativeRenderScene::Talk => Self::Talk,
            NativeRenderScene::TalkActions => Self::TalkActions,
            NativeRenderScene::IncomingCall => Self::IncomingCall,
            NativeRenderScene::OutgoingCall => Self::OutgoingCall,
            NativeRenderScene::InCall => Self::InCall,
            NativeRenderScene::Ask => Self::Ask,
            NativeRenderScene::Power => Self::Power,
            NativeRenderScene::Overlay => Self::Overlay,
        }
    }

    pub const fn registry_scene(self) -> NativeRenderScene {
        match self {
            Self::Hub => NativeRenderScene::Hub,
            Self::Listen => NativeRenderScene::Listen,
            Self::Playlist => NativeRenderScene::Playlist,
            Self::NowPlaying => NativeRenderScene::NowPlaying,
            Self::Talk => NativeRenderScene::Talk,
            Self::TalkActions => NativeRenderScene::TalkActions,
            Self::IncomingCall => NativeRenderScene::IncomingCall,
            Self::OutgoingCall => NativeRenderScene::OutgoingCall,
            Self::InCall => NativeRenderScene::InCall,
            Self::Ask => NativeRenderScene::Ask,
            Self::Power => NativeRenderScene::Power,
            Self::Overlay => NativeRenderScene::Overlay,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hub => "hub",
            Self::Listen => "listen",
            Self::Playlist => "playlist",
            Self::NowPlaying => "now_playing",
            Self::Talk => "talk",
            Self::TalkActions => "talk_actions",
            Self::IncomingCall => "incoming_call",
            Self::OutgoingCall => "outgoing_call",
            Self::InCall => "in_call",
            Self::Ask => "ask",
            Self::Power => "power",
            Self::Overlay => "overlay",
        }
    }
}

pub trait LvglFacade {
    fn create_root(&mut self) -> Result<WidgetId>;

    fn create_container(&mut self, parent: WidgetId, role: &'static str) -> Result<WidgetId>;

    fn create_label(&mut self, parent: WidgetId, role: &'static str) -> Result<WidgetId>;

    fn set_text(&mut self, widget: WidgetId, text: &str) -> Result<()>;

    fn set_selected(&mut self, widget: WidgetId, selected: bool) -> Result<()>;

    fn set_icon(&mut self, widget: WidgetId, icon_key: &str) -> Result<()>;

    fn set_progress(&mut self, widget: WidgetId, value: i32) -> Result<()>;

    fn set_visible(&mut self, widget: WidgetId, visible: bool) -> Result<()>;

    fn set_y(&mut self, widget: WidgetId, y: i32) -> Result<()> {
        let _ = (widget, y);
        Ok(())
    }

    fn set_geometry(
        &mut self,
        widget: WidgetId,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let _ = (widget, x, y, width, height);
        Ok(())
    }

    fn set_variant(
        &mut self,
        widget: WidgetId,
        variant: &'static str,
        accent_rgb: u32,
    ) -> Result<()> {
        let _ = (widget, variant, accent_rgb);
        Ok(())
    }

    fn set_accent(&mut self, widget: WidgetId, rgb: u32) -> Result<()> {
        let _ = (widget, rgb);
        Ok(())
    }

    fn destroy(&mut self, widget: WidgetId) -> Result<()>;
}

impl<T> LvglFacade for Box<T>
where
    T: LvglFacade + ?Sized,
{
    fn create_root(&mut self) -> Result<WidgetId> {
        (**self).create_root()
    }

    fn create_container(&mut self, parent: WidgetId, role: &'static str) -> Result<WidgetId> {
        (**self).create_container(parent, role)
    }

    fn create_label(&mut self, parent: WidgetId, role: &'static str) -> Result<WidgetId> {
        (**self).create_label(parent, role)
    }

    fn set_text(&mut self, widget: WidgetId, text: &str) -> Result<()> {
        (**self).set_text(widget, text)
    }

    fn set_selected(&mut self, widget: WidgetId, selected: bool) -> Result<()> {
        (**self).set_selected(widget, selected)
    }

    fn set_icon(&mut self, widget: WidgetId, icon_key: &str) -> Result<()> {
        (**self).set_icon(widget, icon_key)
    }

    fn set_progress(&mut self, widget: WidgetId, value: i32) -> Result<()> {
        (**self).set_progress(widget, value)
    }

    fn set_visible(&mut self, widget: WidgetId, visible: bool) -> Result<()> {
        (**self).set_visible(widget, visible)
    }

    fn set_y(&mut self, widget: WidgetId, y: i32) -> Result<()> {
        (**self).set_y(widget, y)
    }

    fn set_geometry(
        &mut self,
        widget: WidgetId,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<()> {
        (**self).set_geometry(widget, x, y, width, height)
    }

    fn set_variant(
        &mut self,
        widget: WidgetId,
        variant: &'static str,
        accent_rgb: u32,
    ) -> Result<()> {
        (**self).set_variant(widget, variant, accent_rgb)
    }

    fn set_accent(&mut self, widget: WidgetId, rgb: u32) -> Result<()> {
        (**self).set_accent(widget, rgb)
    }

    fn destroy(&mut self, widget: WidgetId) -> Result<()> {
        (**self).destroy(widget)
    }
}

pub struct LvglRenderer<F> {
    facade: F,
    active_scene: Option<SceneKey>,
    active_screen: Option<UiScreen>,
    controller: Option<Box<dyn ScreenController>>,
}

impl<F> LvglRenderer<F>
where
    F: LvglFacade,
{
    pub fn new(facade: F) -> Self {
        Self {
            facade,
            active_scene: None,
            active_screen: None,
            controller: None,
        }
    }

    pub fn render(&mut self, model: &ScreenModel) -> Result<()> {
        let screen = model.screen();
        let entry = registry::screen_entry(screen);
        let scene = SceneKey::for_screen(screen);

        if self.active_scene != Some(scene) {
            let next = controller_for_entry(entry)?;
            self.clear()?;
            self.controller = Some(next);
            self.active_scene = Some(scene);
        }

        let controller = self
            .controller
            .as_mut()
            .ok_or_else(|| anyhow!("LVGL renderer has no controller for {}", screen.as_str()))?;
        controller.sync(&mut self.facade, model)?;
        self.active_screen = Some(screen);
        Ok(())
    }

    pub fn clear(&mut self) -> Result<()> {
        if let Some(controller) = self.controller.as_mut() {
            controller.teardown(&mut self.facade)?;
        }
        self.controller = None;
        self.active_scene = None;
        self.active_screen = None;
        Ok(())
    }

    pub fn active_scene(&self) -> Option<SceneKey> {
        self.active_scene
    }

    pub fn active_screen(&self) -> Option<UiScreen> {
        self.active_screen
    }

    pub fn facade(&self) -> &F {
        &self.facade
    }

    pub fn facade_mut(&mut self) -> &mut F {
        &mut self.facade
    }
}

fn controller_for_entry(entry: ScreenRegistryEntry) -> Result<Box<dyn ScreenController>> {
    match entry.controller_kind {
        ControllerKind::Hub => Ok(typed_controller(HubController::default())),
        ControllerKind::List => Ok(typed_controller(ListController::default())),
        ControllerKind::Ask => Ok(typed_controller(AskController::default())),
        ControllerKind::TalkActions => Ok(typed_controller(TalkActionsController::default())),
        ControllerKind::NowPlaying => Ok(typed_controller(NowPlayingController::default())),
        ControllerKind::Call => Ok(typed_controller(CallController::default())),
        ControllerKind::Power => Ok(typed_controller(PowerController::default())),
        ControllerKind::Overlay => Ok(typed_controller(OverlayController::default())),
        ControllerKind::Listen | ControllerKind::Playlist | ControllerKind::Talk => {
            anyhow::bail!(
                "generic LVGL renderer cannot use native-only {:?} controller for {}",
                entry.controller_kind,
                entry.screen.as_str()
            )
        }
    }
}

#[cfg(feature = "native-lvgl")]
pub fn open_default_facade(explicit_source: Option<&Path>) -> Result<Box<dyn LvglFacade>> {
    Ok(Box::new(NativeLvglFacade::open(explicit_source)?))
}

#[cfg(not(feature = "native-lvgl"))]
pub fn open_default_facade(_explicit_source: Option<&Path>) -> Result<Box<dyn LvglFacade>> {
    bail!("native-lvgl feature is disabled for this build")
}
