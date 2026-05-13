pub mod screens;
pub mod view_models;

use crate::application::UiView;
use view_models::ScreenModel;
use yoyopod_protocol::ui::UiScreen;
use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot};

pub fn view_for_screen(
    screen: UiScreen,
    snapshot: &RuntimeSnapshot,
    focus_index: usize,
    selected_contact: Option<&ListItemSnapshot>,
) -> UiView {
    match screen {
        UiScreen::TalkContact => {
            screens::call::talk_contact_view(snapshot, focus_index, selected_contact)
        }
        UiScreen::VoiceNote => {
            screens::ask::voice_note_view(snapshot, focus_index, selected_contact)
        }
        UiScreen::Hub => screens::hub::view(snapshot, focus_index),
        UiScreen::Listen => screens::listen::view(snapshot, focus_index),
        UiScreen::Playlists => screens::music::playlists_view(snapshot, focus_index),
        UiScreen::RecentTracks => screens::music::recent_tracks_view(snapshot, focus_index),
        UiScreen::NowPlaying => screens::music::now_playing_view(snapshot, focus_index),
        UiScreen::Ask => screens::ask::ask_view(snapshot, focus_index),
        UiScreen::Talk => screens::talk::view(focus_index),
        UiScreen::Contacts => screens::call::contacts_view(snapshot, focus_index),
        UiScreen::CallHistory => screens::call::call_history_view(snapshot, focus_index),
        UiScreen::IncomingCall => screens::call::incoming_view(snapshot, focus_index),
        UiScreen::OutgoingCall => screens::call::outgoing_view(snapshot, focus_index),
        UiScreen::InCall => screens::call::in_call_view(snapshot, focus_index),
        UiScreen::Power => screens::power::view(snapshot, focus_index),
        UiScreen::Loading => screens::overlay::loading_view(snapshot),
        UiScreen::Error => screens::overlay::error_view(snapshot),
    }
}

pub fn screen_model_for_screen(
    screen: UiScreen,
    snapshot: &RuntimeSnapshot,
    focus_index: usize,
    selected_contact: Option<&ListItemSnapshot>,
) -> ScreenModel {
    match screen {
        UiScreen::TalkContact => ScreenModel::TalkContact(screens::call::talk_contact_model(
            snapshot,
            focus_index,
            selected_contact,
        )),
        UiScreen::VoiceNote => ScreenModel::VoiceNote(screens::ask::voice_note_model(
            snapshot,
            focus_index,
            selected_contact,
        )),
        UiScreen::Hub => ScreenModel::Hub(screens::hub::model(snapshot, focus_index)),
        UiScreen::Listen => ScreenModel::Listen(screens::listen::model(snapshot, focus_index)),
        UiScreen::Playlists => {
            ScreenModel::Playlists(screens::music::playlists_model(snapshot, focus_index))
        }
        UiScreen::RecentTracks => {
            ScreenModel::RecentTracks(screens::music::recent_tracks_model(snapshot, focus_index))
        }
        UiScreen::NowPlaying => {
            ScreenModel::NowPlaying(screens::music::now_playing_model(snapshot))
        }
        UiScreen::Ask => ScreenModel::Ask(screens::ask::ask_model(snapshot)),
        UiScreen::Talk => ScreenModel::Talk(screens::talk::model(snapshot, focus_index)),
        UiScreen::Contacts => {
            ScreenModel::Contacts(screens::call::contacts_model(snapshot, focus_index))
        }
        UiScreen::CallHistory => {
            ScreenModel::CallHistory(screens::call::call_history_model(snapshot, focus_index))
        }
        UiScreen::IncomingCall => {
            ScreenModel::IncomingCall(screens::call::incoming_model(snapshot))
        }
        UiScreen::OutgoingCall => {
            ScreenModel::OutgoingCall(screens::call::outgoing_model(snapshot))
        }
        UiScreen::InCall => ScreenModel::InCall(screens::call::in_call_model(snapshot)),
        UiScreen::Power => ScreenModel::Power(screens::power::model(snapshot, focus_index)),
        UiScreen::Loading => ScreenModel::Loading(screens::overlay::loading_model(snapshot)),
        UiScreen::Error => ScreenModel::Error(screens::overlay::error_model(snapshot)),
    }
}
