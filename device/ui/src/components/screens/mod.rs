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
        UiScreen::NowPlaying => now_playing::scene(snapshot),
        UiScreen::Ask => ask::scene(snapshot, focus),
        UiScreen::Talk => talk::scene(focus),
        UiScreen::Contacts => contacts::scene(&contacts::props_from(snapshot, focus)),
        UiScreen::CallHistory => call_history::scene(&call_history::props_from(snapshot, focus)),
        UiScreen::TalkContact => talk_contact::scene(snapshot, focus, selected_contact),
        UiScreen::VoiceNote => voice_note::scene(snapshot, focus),
        UiScreen::IncomingCall => incoming_call::scene(snapshot),
        UiScreen::OutgoingCall => outgoing_call::scene(snapshot),
        UiScreen::InCall => in_call::scene(snapshot),
        UiScreen::Power => power::scene(snapshot, focus),
        UiScreen::Loading => loading::scene(snapshot),
        UiScreen::Error => error::scene(snapshot),
    }
}
