use crate::runtime::RuntimeSnapshot;

use super::UiScreen;

pub fn screen_for_app_state(app_state: &str) -> Option<UiScreen> {
    match app_state.trim() {
        "hub" | "home" | "menu" => Some(UiScreen::Hub),
        "listen" => Some(UiScreen::Listen),
        "playlists" => Some(UiScreen::Playlists),
        "recent_tracks" => Some(UiScreen::RecentTracks),
        "now_playing" => Some(UiScreen::NowPlaying),
        "ask" => Some(UiScreen::Ask),
        "call" | "talk" => Some(UiScreen::Talk),
        "contacts" => Some(UiScreen::Contacts),
        "call_history" => Some(UiScreen::CallHistory),
        "talk_contact" => Some(UiScreen::TalkContact),
        "voice_note" => Some(UiScreen::VoiceNote),
        "incoming_call" => Some(UiScreen::IncomingCall),
        "outgoing_call" => Some(UiScreen::OutgoingCall),
        "in_call" => Some(UiScreen::InCall),
        "power" | "setup" => Some(UiScreen::Power),
        "loading" => Some(UiScreen::Loading),
        "error" => Some(UiScreen::Error),
        _ => None,
    }
}

pub fn runtime_preemption(snapshot: &RuntimeSnapshot) -> Option<UiScreen> {
    if !snapshot.overlay.error.trim().is_empty() {
        return Some(UiScreen::Error);
    }
    if snapshot.overlay.loading {
        return Some(UiScreen::Loading);
    }
    match snapshot.call.state.as_str() {
        "incoming" => Some(UiScreen::IncomingCall),
        "outgoing" => Some(UiScreen::OutgoingCall),
        "active" => Some(UiScreen::InCall),
        _ => None,
    }
}

pub fn is_call_screen(screen: UiScreen) -> bool {
    matches!(
        screen,
        UiScreen::IncomingCall | UiScreen::OutgoingCall | UiScreen::InCall
    )
}

pub fn is_overlay_screen(screen: UiScreen) -> bool {
    matches!(screen, UiScreen::Loading | UiScreen::Error)
}
