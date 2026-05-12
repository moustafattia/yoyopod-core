use yoyopod_protocol::ui::RuntimeSnapshot;

use super::UiScreen;

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
