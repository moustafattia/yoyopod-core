pub mod ask;
pub mod call_history;
pub mod chrome;
pub mod common;
pub mod contacts;
pub mod error;
pub mod hub;
pub mod in_call;
pub mod incoming_call;
pub mod listen;
pub mod loading;
pub mod now_playing;
pub mod outgoing_call;
pub mod playlists;
pub mod power;
pub mod recent_tracks;
pub mod talk;
pub mod talk_contact;
pub mod voice_note;

use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::scene::Scene;

pub fn scene_for_screen(
    screen: UiScreen,
    snapshot: &RuntimeSnapshot,
    focus: usize,
    selected_contact: Option<&ListItemSnapshot>,
) -> Scene {
    match screen {
        UiScreen::Hub => hub::scene(&hub::props_from(snapshot, focus)),
        UiScreen::Listen => listen::scene(&listen::props_from(snapshot, focus)),
        UiScreen::Playlists => playlists::scene(&playlists::props_from(snapshot, focus)),
        UiScreen::RecentTracks => recent_tracks::scene(&recent_tracks::props_from(snapshot, focus)),
        UiScreen::NowPlaying => now_playing::scene(&now_playing::props_from(snapshot)),
        UiScreen::Ask => ask::scene(&ask::props_from(snapshot, focus)),
        UiScreen::Talk => talk::scene(&talk::props_from(focus)),
        UiScreen::Contacts => contacts::scene(&contacts::props_from(snapshot, focus)),
        UiScreen::CallHistory => call_history::scene(&call_history::props_from(snapshot, focus)),
        UiScreen::TalkContact => {
            talk_contact::scene(&talk_contact::props_from(snapshot, focus, selected_contact))
        }
        UiScreen::VoiceNote => voice_note::scene(&voice_note::props_from(snapshot, focus)),
        UiScreen::IncomingCall => incoming_call::scene(&incoming_call::props_from(snapshot)),
        UiScreen::OutgoingCall => outgoing_call::scene(&outgoing_call::props_from(snapshot)),
        UiScreen::InCall => in_call::scene(&in_call::props_from(snapshot)),
        UiScreen::Power => power::scene(&power::props_from(snapshot, focus)),
        UiScreen::Loading => loading::scene(&loading::props_from(snapshot)),
        UiScreen::Error => error::scene(&error::props_from(snapshot)),
    }
}
