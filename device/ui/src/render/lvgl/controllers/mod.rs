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

use crate::presentation::screens::ScreenModel;
use crate::presentation::transitions::TransitionSampler;
use crate::render::lvgl::LvglFacade;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{UiRuntime, UiScreen};
    use crate::presentation::screens::{
        CallViewModel, ChromeModel, HubViewModel, ListRowModel, ListScreenModel,
        NowPlayingViewModel, OverlayViewModel, ScreenModel, StatusBarModel,
    };
    use crate::presentation::transitions::TransitionSampler;
    use crate::render::lvgl::WidgetId;
    use yoyopod_protocol::ui::AnimationRequest;

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

    #[test]
    fn animate_tick_drives_controller_opacity_calls() {
        let mut runtime = UiRuntime::default();
        runtime.mark_clean();
        runtime.start_animation(
            AnimationRequest {
                transition_id: "screen_fade".to_string(),
                duration_ms: 200,
            },
            1000,
        );
        runtime.mark_clean();

        assert!(runtime.advance_animations(1100));
        assert!(runtime.dirty_state().animation);

        let sampler = TransitionSampler::new(runtime.active_transitions(), 1100);
        let mut facade = RecordingFacade::default();
        let mut controller = HubController::default();
        let model = hub_model();

        controller
            .sync_model(&mut facade, HubController::model(&model).unwrap(), &sampler)
            .unwrap();

        assert!(
            facade
                .opacity_calls
                .iter()
                .any(|(_, opacity)| *opacity > 0 && *opacity < 255),
            "screen fade transition must reach facade opacity calls"
        );
    }

    #[test]
    fn animate_tick_drives_selected_row_offset_calls() {
        let mut runtime = UiRuntime {
            active_screen: UiScreen::Listen,
            focus_index: 1,
            ..UiRuntime::default()
        };
        runtime.mark_clean();
        runtime.start_animation(
            AnimationRequest {
                transition_id: "selection_move".to_string(),
                duration_ms: 200,
            },
            1000,
        );
        runtime.mark_clean();

        assert!(runtime.advance_animations(1100));
        assert!(runtime.dirty_state().animation);

        let sampler = TransitionSampler::new(runtime.active_transitions(), 1100);
        let mut facade = RecordingFacade::default();
        let mut controller = ListController::default();
        let list = list_model_with_rows();
        let model = ScreenModel::Listen(list);

        controller
            .sync_model(
                &mut facade,
                ListController::model(&model).unwrap(),
                &sampler,
            )
            .unwrap();

        assert!(
            facade.y_offset_calls.iter().any(|(_, offset)| *offset > 0),
            "selection transition must reach facade y-offset calls"
        );
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

    fn list_model_with_rows() -> ListScreenModel {
        ListScreenModel {
            chrome: chrome(),
            title: "Listen".to_string(),
            subtitle: String::new(),
            rows: vec![
                ListRowModel {
                    id: "one".to_string(),
                    title: "One".to_string(),
                    subtitle: String::new(),
                    icon_key: "listen".to_string(),
                    selected: false,
                },
                ListRowModel {
                    id: "two".to_string(),
                    title: "Two".to_string(),
                    subtitle: String::new(),
                    icon_key: "listen".to_string(),
                    selected: true,
                },
            ],
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

    #[derive(Default)]
    struct RecordingFacade {
        next_id: u64,
        opacity_calls: Vec<(WidgetId, u8)>,
        y_offset_calls: Vec<(WidgetId, i32)>,
    }

    impl RecordingFacade {
        fn next_widget(&mut self) -> WidgetId {
            let id = WidgetId::new(self.next_id);
            self.next_id += 1;
            id
        }
    }

    impl LvglFacade for RecordingFacade {
        fn create_root(&mut self) -> Result<WidgetId> {
            Ok(self.next_widget())
        }

        fn create_container(&mut self, _parent: WidgetId, _role: &'static str) -> Result<WidgetId> {
            Ok(self.next_widget())
        }

        fn create_label(&mut self, _parent: WidgetId, _role: &'static str) -> Result<WidgetId> {
            Ok(self.next_widget())
        }

        fn set_text(&mut self, _widget: WidgetId, _text: &str) -> Result<()> {
            Ok(())
        }

        fn set_selected(&mut self, _widget: WidgetId, _selected: bool) -> Result<()> {
            Ok(())
        }

        fn set_icon(&mut self, _widget: WidgetId, _icon_key: &str) -> Result<()> {
            Ok(())
        }

        fn set_progress(&mut self, _widget: WidgetId, _value: i32) -> Result<()> {
            Ok(())
        }

        fn set_visible(&mut self, _widget: WidgetId, _visible: bool) -> Result<()> {
            Ok(())
        }

        fn set_opacity(&mut self, widget: WidgetId, opacity: u8) -> Result<()> {
            self.opacity_calls.push((widget, opacity));
            Ok(())
        }

        fn set_y_offset(&mut self, widget: WidgetId, offset: i32) -> Result<()> {
            self.y_offset_calls.push((widget, offset));
            Ok(())
        }

        fn destroy(&mut self, _widget: WidgetId) -> Result<()> {
            Ok(())
        }
    }
}
