mod ask;
mod call;
mod hub;
mod list;
mod listen;
mod now_playing;
mod overlay;
mod playlist;
mod power;
mod shared;
mod talk;
mod talk_actions;

use anyhow::Result;

use crate::render::lvgl::LvglFacade;
use crate::screens::ScreenModel;

pub use ask::AskController;
pub use call::CallController;
pub use hub::HubController;
pub use list::ListController;
pub use listen::ListenController;
pub use now_playing::NowPlayingController;
pub use overlay::OverlayController;
pub use playlist::PlaylistController;
pub use power::PowerController;
pub use talk::TalkController;
pub use talk_actions::TalkActionsController;

pub trait ScreenController {
    fn sync(&mut self, facade: &mut dyn LvglFacade, model: &ScreenModel) -> Result<()>;

    fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()>;
}

pub trait TypedScreenController {
    type Model<'a>
    where
        Self: 'a;

    fn model<'a>(model: &'a ScreenModel) -> Result<Self::Model<'a>>;

    fn sync_model(&mut self, facade: &mut dyn LvglFacade, model: Self::Model<'_>) -> Result<()>;

    fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()>;
}

pub struct ControllerAdapter<C> {
    controller: C,
}

impl<C> ControllerAdapter<C> {
    pub fn new(controller: C) -> Self {
        Self { controller }
    }
}

impl<C> ScreenController for ControllerAdapter<C>
where
    C: TypedScreenController,
{
    fn sync(&mut self, facade: &mut dyn LvglFacade, model: &ScreenModel) -> Result<()> {
        let model = C::model(model)?;
        self.controller.sync_model(facade, model)
    }

    fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
        self.controller.teardown(facade)
    }
}

pub fn typed_controller<C>(controller: C) -> Box<dyn ScreenController>
where
    C: TypedScreenController + 'static,
{
    Box::new(ControllerAdapter::new(controller))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::{
        CallViewModel, ChromeModel, HubViewModel, ListScreenModel, NowPlayingViewModel,
        OverlayViewModel, ScreenModel, StatusBarModel,
    };

    #[test]
    fn typed_controllers_accept_declared_models_only() {
        let hub = hub_model();
        let overlay = ScreenModel::Loading(overlay_model());
        let call = ScreenModel::IncomingCall(call_model());
        let now_playing = ScreenModel::NowPlaying(now_playing_model());
        let list = ScreenModel::Listen(list_model());

        assert!(HubController::model(&hub).is_ok());
        assert!(HubController::model(&overlay).is_err());

        assert!(OverlayController::model(&overlay).is_ok());
        assert!(OverlayController::model(&hub).is_err());

        assert!(CallController::model(&call).is_ok());
        assert!(CallController::model(&now_playing).is_err());

        assert!(ListController::model(&list).is_ok());
        assert!(ListController::model(&overlay).is_err());
    }

    fn chrome() -> ChromeModel {
        ChromeModel {
            status: StatusBarModel {
                network_connected: false,
                network_enabled: false,
                connection_type: String::new(),
                signal_strength: 0,
                gps_has_fix: false,
                battery_percent: 100,
                charging: false,
                power_available: true,
                voip_state: 1,
            },
            footer: String::new(),
        }
    }

    fn hub_model() -> ScreenModel {
        ScreenModel::Hub(HubViewModel {
            chrome: chrome(),
            cards: Vec::new(),
            selected_index: 0,
        })
    }

    fn list_model() -> ListScreenModel {
        ListScreenModel {
            chrome: chrome(),
            title: String::new(),
            subtitle: String::new(),
            rows: Vec::new(),
        }
    }

    fn now_playing_model() -> NowPlayingViewModel {
        NowPlayingViewModel {
            chrome: chrome(),
            title: String::new(),
            artist: String::new(),
            state_text: String::new(),
            progress_permille: 0,
        }
    }

    fn call_model() -> CallViewModel {
        CallViewModel {
            chrome: chrome(),
            title: String::new(),
            subtitle: String::new(),
            detail: String::new(),
            muted: false,
        }
    }

    fn overlay_model() -> OverlayViewModel {
        OverlayViewModel {
            chrome: chrome(),
            title: String::new(),
            subtitle: String::new(),
        }
    }
}
