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

use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::scene::Scene;

pub fn scene_for_screen(screen: UiScreen, snapshot: &RuntimeSnapshot, focus: usize) -> Scene {
    match screen {
        UiScreen::Hub => hub::scene(&hub::props_from(snapshot, focus)),
        UiScreen::Listen => listen::scene(&listen::props_from(snapshot, focus)),
        UiScreen::Playlists => playlists::scene(snapshot, focus),
        UiScreen::RecentTracks => recent_tracks::scene(snapshot, focus),
        UiScreen::NowPlaying => now_playing::scene(snapshot),
        UiScreen::Ask => ask::scene(focus),
        UiScreen::Talk => talk::scene(focus),
        UiScreen::Contacts => contacts::scene(snapshot, focus),
        UiScreen::CallHistory => call_history::scene(snapshot, focus),
        UiScreen::TalkContact => talk_contact::scene(focus),
        UiScreen::VoiceNote => voice_note::scene(focus),
        UiScreen::IncomingCall => incoming_call::scene(),
        UiScreen::OutgoingCall => outgoing_call::scene(),
        UiScreen::InCall => in_call::scene(),
        UiScreen::Power => power::scene(focus),
        UiScreen::Loading => loading::scene(snapshot),
        UiScreen::Error => error::scene(snapshot),
    }
}
