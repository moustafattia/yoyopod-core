use anyhow::{bail, Result};

use crate::presentation::registry::{self, ControllerKind, NativeRenderScene};
use crate::presentation::transitions::TransitionSampler;
use crate::presentation::view_models::ScreenModel;
use crate::render::screens::{
    AskController, CallController, CallControllerModel, HubController, ListenController,
    NowPlayingController, OverlayController, PlaylistController, PlaylistControllerModel,
    PowerController, TalkActionsController, TalkController, TypedScreenController,
};
use crate::render::widgets::LvglFacade;
use yoyopod_protocol::ui::UiScreen;

pub(super) enum NativeSceneController {
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
    pub(super) fn sync(
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

    pub(super) fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
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

pub(super) fn controller_for_native_scene(
    scene: NativeRenderScene,
) -> Result<NativeSceneController> {
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
