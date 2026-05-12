use std::path::Path;

use anyhow::{bail, Result};

use crate::runtime::UiScreen;
use crate::screens::{AskViewModel, ChromeModel, ListScreenModel, ScreenModel, StatusBarModel};

use super::{
    AskController, CallController, HubController, ListenController, LvglFacade, NativeSceneKey,
    NowPlayingController, PlaylistController, PowerController, ScreenController,
    TalkActionsController, TalkController,
};

pub trait SceneBridge {
    fn build_scene(&mut self, scene: NativeSceneKey) -> Result<()>;
    fn sync_status(&mut self, status: &StatusBarModel) -> Result<()>;
    fn sync_scene(&mut self, model: &ScreenModel) -> Result<()>;
    fn destroy_scene(&mut self, scene: NativeSceneKey);
    fn clear_screen(&mut self) -> Result<()>;
}

pub struct NativeSceneRenderer<B> {
    bridge: B,
    active_scene: Option<NativeSceneKey>,
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

    pub fn render(&mut self, model: &ScreenModel) -> Result<()> {
        let screen = model.screen();
        let scene = NativeSceneKey::for_screen(screen);

        if self.active_scene != Some(scene) {
            if let Some(active_scene) = self.active_scene.take() {
                self.bridge.destroy_scene(active_scene);
            }
            self.bridge.build_scene(scene)?;
            self.active_scene = Some(scene);
        }

        self.bridge.sync_status(chrome(model).status())?;
        self.bridge.sync_scene(model)?;
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

    pub fn active_scene(&self) -> Option<NativeSceneKey> {
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
    controller: Option<Box<dyn ScreenController>>,
    active_scene: Option<NativeSceneKey>,
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

    pub fn active_scene(&self) -> Option<NativeSceneKey> {
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
    fn build_scene(&mut self, scene: NativeSceneKey) -> Result<()> {
        if self.active_scene == Some(scene) {
            return Ok(());
        }
        if self.active_scene.is_some() {
            if let Some(controller) = self.controller.as_mut() {
                let _ = controller.teardown(&mut self.facade);
            }
        }
        self.controller = Some(controller_for_native_scene(scene));
        self.active_scene = Some(scene);
        Ok(())
    }

    fn sync_status(&mut self, status: &StatusBarModel) -> Result<()> {
        self.last_status = Some(status.clone());
        Ok(())
    }

    fn sync_scene(&mut self, model: &ScreenModel) -> Result<()> {
        let expected_scene = NativeSceneKey::for_screen(model.screen());
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
        controller.sync(&mut self.facade, &model)
    }

    fn destroy_scene(&mut self, scene: NativeSceneKey) {
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

fn controller_for_native_scene(scene: NativeSceneKey) -> Box<dyn ScreenController> {
    match scene {
        NativeSceneKey::Hub => Box::new(HubController::default()),
        NativeSceneKey::Listen => Box::new(ListenController::default()),
        NativeSceneKey::Playlist => Box::new(PlaylistController::default()),
        NativeSceneKey::NowPlaying => Box::new(NowPlayingController::default()),
        NativeSceneKey::Talk => Box::new(TalkController::default()),
        NativeSceneKey::TalkActions => Box::new(TalkActionsController::default()),
        NativeSceneKey::IncomingCall | NativeSceneKey::OutgoingCall | NativeSceneKey::InCall => {
            Box::new(CallController::default())
        }
        NativeSceneKey::Ask => Box::new(AskController::default()),
        NativeSceneKey::Power => Box::new(PowerController::default()),
        NativeSceneKey::Overlay => Box::new(AskController::default()),
    }
}

#[cfg(feature = "native-lvgl")]
impl RustSceneBridge<super::NativeLvglFacade> {
    pub fn open(explicit_source: Option<&Path>) -> Result<Self> {
        Ok(Self::new(super::NativeLvglFacade::open(explicit_source)?))
    }

    pub fn display_needs_reset(&self, framebuffer: &crate::framebuffer::Framebuffer) -> bool {
        self.facade().display_needs_reset(framebuffer)
    }

    pub fn ensure_display_registered(
        &mut self,
        framebuffer: &crate::framebuffer::Framebuffer,
    ) -> Result<()> {
        self.facade_mut().ensure_display_registered(framebuffer)
    }

    pub fn render_frame(
        &mut self,
        framebuffer: &mut crate::framebuffer::Framebuffer,
    ) -> Result<()> {
        self.facade_mut().render_frame(framebuffer)
    }
}

const NATIVE_LIST_VISIBLE_ROWS: usize = 4;

#[derive(Debug, Clone, Copy)]
enum NativeListSelection {
    Wrap,
    Clamp,
}

fn rust_owned_scene_model(model: &ScreenModel, scene: NativeSceneKey) -> ScreenModel {
    match (scene, model) {
        (NativeSceneKey::Listen, ScreenModel::Listen(list)) => {
            ScreenModel::Listen(capped_list_model(list, NativeListSelection::Wrap))
        }
        (NativeSceneKey::Playlist, ScreenModel::Playlists(list)) => {
            ScreenModel::Playlists(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeSceneKey::Playlist, ScreenModel::RecentTracks(list)) => {
            ScreenModel::RecentTracks(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeSceneKey::Playlist, ScreenModel::Contacts(list)) => {
            ScreenModel::Contacts(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeSceneKey::Playlist, ScreenModel::CallHistory(list)) => {
            ScreenModel::CallHistory(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeSceneKey::Overlay, ScreenModel::Loading(overlay))
        | (NativeSceneKey::Overlay, ScreenModel::Error(overlay)) => {
            ScreenModel::Ask(AskViewModel {
                chrome: overlay.chrome.clone(),
                title: overlay.title.clone(),
                subtitle: overlay.subtitle.clone(),
                icon_key: "ask".to_string(),
            })
        }
        _ => model.clone(),
    }
}

fn capped_list_model(model: &ListScreenModel, selection: NativeListSelection) -> ListScreenModel {
    let mut rows = model
        .rows
        .iter()
        .take(NATIVE_LIST_VISIBLE_ROWS)
        .cloned()
        .collect::<Vec<_>>();

    if !rows.is_empty() {
        let selected_index = model.rows.iter().position(|row| row.selected).unwrap_or(0);
        let visible_index = match selection {
            NativeListSelection::Wrap => selected_index % rows.len(),
            NativeListSelection::Clamp => selected_index.min(rows.len() - 1),
        };
        for row in &mut rows {
            row.selected = false;
        }
        rows[visible_index].selected = true;
    }

    ListScreenModel {
        chrome: model.chrome.clone(),
        title: model.title.clone(),
        subtitle: model.subtitle.clone(),
        rows,
    }
}

trait ChromeRef {
    fn status(&self) -> &StatusBarModel;
}

impl ChromeRef for ChromeModel {
    fn status(&self) -> &StatusBarModel {
        &self.status
    }
}

fn chrome(model: &ScreenModel) -> &ChromeModel {
    match model {
        ScreenModel::Hub(model) => &model.chrome,
        ScreenModel::Listen(model)
        | ScreenModel::Playlists(model)
        | ScreenModel::RecentTracks(model)
        | ScreenModel::Talk(model)
        | ScreenModel::Contacts(model)
        | ScreenModel::CallHistory(model) => &model.chrome,
        ScreenModel::NowPlaying(model) => &model.chrome,
        ScreenModel::Ask(model) => &model.chrome,
        ScreenModel::TalkContact(model) | ScreenModel::VoiceNote(model) => &model.chrome,
        ScreenModel::IncomingCall(model)
        | ScreenModel::OutgoingCall(model)
        | ScreenModel::InCall(model) => &model.chrome,
        ScreenModel::Power(model) => &model.chrome,
        ScreenModel::Loading(model) | ScreenModel::Error(model) => &model.chrome,
    }
}
