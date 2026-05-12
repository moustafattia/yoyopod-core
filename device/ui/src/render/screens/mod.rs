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
mod status_bar;
mod talk;
mod talk_actions;

use anyhow::Result;

use crate::presentation::transitions::TransitionSampler;
use crate::presentation::view_models::ScreenModel;
use crate::render::widgets::LvglFacade;

pub use ask::AskController;
pub use call::{CallController, CallControllerModel};
pub use hub::HubController;
pub use list::ListController;
pub use listen::ListenController;
pub use now_playing::NowPlayingController;
pub use overlay::OverlayController;
pub use playlist::{PlaylistController, PlaylistControllerModel};
pub use power::PowerController;
pub use talk::TalkController;
pub use talk_actions::TalkActionsController;

pub trait TypedScreenController {
    const SUPPORTS_TRANSITIONS: bool = false;

    type Model<'a>
    where
        Self: 'a;

    fn model<'a>(model: &'a ScreenModel) -> Result<Self::Model<'a>>;

    fn sync_model(
        &mut self,
        facade: &mut dyn LvglFacade,
        model: Self::Model<'_>,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()>;

    fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()>;
}
