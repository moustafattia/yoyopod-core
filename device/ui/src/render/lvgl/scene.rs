#[cfg(feature = "native-lvgl")]
use std::path::Path;

use anyhow::{bail, Result};

use crate::app::UiScreen;
use crate::presentation::registry::{self, ControllerKind, NativeRenderScene};
use crate::presentation::screens::{ListScreenModel, ScreenModel, StatusBarModel};
use crate::presentation::transitions::TransitionSampler;

use crate::render::lvgl::controllers::{
    AskController, CallController, CallControllerModel, HubController, ListenController,
    NowPlayingController, OverlayController, PlaylistController, PlaylistControllerModel,
    PowerController, TalkActionsController, TalkController, TypedScreenController,
};
use crate::render::lvgl::LvglFacade;

const fn native_scene_for_screen(screen: UiScreen) -> NativeRenderScene {
    registry::screen_entry(screen).native_scene
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

enum NativeSceneController {
    Hub(HubController),
    Listen(ListenController),
    Playlist(PlaylistController),
    NowPlaying(NowPlayingController),
    Talk(TalkController),
    TalkActions(TalkActionsController),
    Call(CallController),
    Ask(AskController),
    Power(PowerController),
    Overlay(OverlayController),
}

impl NativeSceneController {
    fn sync(
        &mut self,
        facade: &mut dyn LvglFacade,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        match (self, model) {
            (Self::Hub(controller), ScreenModel::Hub(model)) => {
                controller.sync_model(facade, model, transitions)
            }
            (Self::Listen(controller), ScreenModel::Listen(model)) => {
                controller.sync_model(facade, model, transitions)
            }
            (Self::Playlist(controller), ScreenModel::Playlists(model)) => controller.sync_model(
                facade,
                PlaylistControllerModel {
                    screen: UiScreen::Playlists,
                    list: model,
                },
                transitions,
            ),
            (Self::Playlist(controller), ScreenModel::RecentTracks(model)) => controller
                .sync_model(
                    facade,
                    PlaylistControllerModel {
                        screen: UiScreen::RecentTracks,
                        list: model,
                    },
                    transitions,
                ),
            (Self::Playlist(controller), ScreenModel::Contacts(model)) => controller.sync_model(
                facade,
                PlaylistControllerModel {
                    screen: UiScreen::Contacts,
                    list: model,
                },
                transitions,
            ),
            (Self::Playlist(controller), ScreenModel::CallHistory(model)) => controller.sync_model(
                facade,
                PlaylistControllerModel {
                    screen: UiScreen::CallHistory,
                    list: model,
                },
                transitions,
            ),
            (Self::NowPlaying(controller), ScreenModel::NowPlaying(model)) => {
                controller.sync_model(facade, model, transitions)
            }
            (Self::Talk(controller), ScreenModel::Talk(model)) => {
                controller.sync_model(facade, model, transitions)
            }
            (Self::TalkActions(controller), ScreenModel::TalkContact(model))
            | (Self::TalkActions(controller), ScreenModel::VoiceNote(model)) => {
                controller.sync_model(facade, model, transitions)
            }
            (Self::Call(controller), ScreenModel::IncomingCall(model)) => controller.sync_model(
                facade,
                CallControllerModel {
                    screen: UiScreen::IncomingCall,
                    call: model,
                },
                transitions,
            ),
            (Self::Call(controller), ScreenModel::OutgoingCall(model)) => controller.sync_model(
                facade,
                CallControllerModel {
                    screen: UiScreen::OutgoingCall,
                    call: model,
                },
                transitions,
            ),
            (Self::Call(controller), ScreenModel::InCall(model)) => controller.sync_model(
                facade,
                CallControllerModel {
                    screen: UiScreen::InCall,
                    call: model,
                },
                transitions,
            ),
            (Self::Ask(controller), ScreenModel::Ask(model)) => {
                controller.sync_model(facade, model, transitions)
            }
            (Self::Power(controller), ScreenModel::Power(model)) => {
                controller.sync_model(facade, model, transitions)
            }
            (Self::Overlay(controller), ScreenModel::Loading(model))
            | (Self::Overlay(controller), ScreenModel::Error(model)) => {
                controller.sync_model(facade, model, transitions)
            }
            (controller, model) => bail!(
                "native LVGL controller {} received {} model",
                controller.kind_name(),
                model.screen().as_str()
            ),
        }
    }

    fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
        match self {
            Self::Hub(controller) => TypedScreenController::teardown(controller, facade),
            Self::Listen(controller) => TypedScreenController::teardown(controller, facade),
            Self::Playlist(controller) => TypedScreenController::teardown(controller, facade),
            Self::NowPlaying(controller) => TypedScreenController::teardown(controller, facade),
            Self::Talk(controller) => TypedScreenController::teardown(controller, facade),
            Self::TalkActions(controller) => TypedScreenController::teardown(controller, facade),
            Self::Call(controller) => TypedScreenController::teardown(controller, facade),
            Self::Ask(controller) => TypedScreenController::teardown(controller, facade),
            Self::Power(controller) => TypedScreenController::teardown(controller, facade),
            Self::Overlay(controller) => TypedScreenController::teardown(controller, facade),
        }
    }

    const fn kind_name(&self) -> &'static str {
        match self {
            Self::Hub(_) => "hub",
            Self::Listen(_) => "listen",
            Self::Playlist(_) => "playlist",
            Self::NowPlaying(_) => "now_playing",
            Self::Talk(_) => "talk",
            Self::TalkActions(_) => "talk_actions",
            Self::Call(_) => "call",
            Self::Ask(_) => "ask",
            Self::Power(_) => "power",
            Self::Overlay(_) => "overlay",
        }
    }
}

fn controller_for_native_scene(scene: NativeRenderScene) -> Result<NativeSceneController> {
    let kind = registry::controller_kind_for_native_scene(scene);
    match kind {
        ControllerKind::Hub => Ok(NativeSceneController::Hub(HubController::default())),
        ControllerKind::Listen => Ok(NativeSceneController::Listen(ListenController::default())),
        ControllerKind::Playlist => Ok(NativeSceneController::Playlist(
            PlaylistController::default(),
        )),
        ControllerKind::NowPlaying => Ok(NativeSceneController::NowPlaying(
            NowPlayingController::default(),
        )),
        ControllerKind::Talk => Ok(NativeSceneController::Talk(TalkController::default())),
        ControllerKind::TalkActions => Ok(NativeSceneController::TalkActions(
            TalkActionsController::default(),
        )),
        ControllerKind::Call => Ok(NativeSceneController::Call(CallController::default())),
        ControllerKind::Ask => Ok(NativeSceneController::Ask(AskController::default())),
        ControllerKind::Power => Ok(NativeSceneController::Power(PowerController::default())),
        ControllerKind::Overlay => Ok(NativeSceneController::Overlay(OverlayController::default())),
        ControllerKind::List => bail!(
            "native LVGL scene {} resolved unsupported generic List controller",
            scene.as_str()
        ),
    }
}

#[cfg(feature = "native-lvgl")]
impl RustSceneBridge<crate::render::lvgl::backend::NativeLvglFacade> {
    pub fn open(explicit_source: Option<&Path>) -> Result<Self> {
        Ok(Self::new(
            crate::render::lvgl::backend::NativeLvglFacade::open(explicit_source)?,
        ))
    }

    pub fn display_needs_reset(&self, framebuffer: &crate::render::Framebuffer) -> bool {
        self.facade().display_needs_reset(framebuffer)
    }

    pub fn ensure_display_registered(
        &mut self,
        framebuffer: &crate::render::Framebuffer,
    ) -> Result<()> {
        self.facade_mut().ensure_display_registered(framebuffer)
    }

    pub fn render_frame(&mut self, framebuffer: &mut crate::render::Framebuffer) -> Result<()> {
        self.facade_mut().render_frame(framebuffer)
    }
}

const NATIVE_LIST_VISIBLE_ROWS: usize = 4;

#[derive(Debug, Clone, Copy)]
enum NativeListSelection {
    Wrap,
    Clamp,
}

fn rust_owned_scene_model(model: &ScreenModel, scene: NativeRenderScene) -> ScreenModel {
    match (scene, model) {
        (NativeRenderScene::Listen, ScreenModel::Listen(list)) => {
            ScreenModel::Listen(capped_list_model(list, NativeListSelection::Wrap))
        }
        (NativeRenderScene::Playlist, ScreenModel::Playlists(list)) => {
            ScreenModel::Playlists(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeRenderScene::Playlist, ScreenModel::RecentTracks(list)) => {
            ScreenModel::RecentTracks(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeRenderScene::Playlist, ScreenModel::Contacts(list)) => {
            ScreenModel::Contacts(capped_list_model(list, NativeListSelection::Clamp))
        }
        (NativeRenderScene::Playlist, ScreenModel::CallHistory(list)) => {
            ScreenModel::CallHistory(capped_list_model(list, NativeListSelection::Clamp))
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
