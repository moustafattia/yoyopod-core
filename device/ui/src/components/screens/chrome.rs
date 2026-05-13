use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::components::widgets::{footer_bar, status_bar, FooterBarProps, StatusBarProps};
use crate::engine::{Element, Key};
use crate::render_contract::ElementKind;
use crate::scene::roles;
use crate::scene::{HudScene, HudStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenChrome {
    pub title: String,
    pub status: HudStatus,
    pub footer_text: String,
}

pub fn chrome_for_screen(
    screen: UiScreen,
    snapshot: &RuntimeSnapshot,
    focus_index: usize,
    selected_contact: Option<&ListItemSnapshot>,
) -> ScreenChrome {
    ScreenChrome {
        title: title_for_screen(screen, snapshot, focus_index, selected_contact),
        status: status_from_snapshot(snapshot),
        footer_text: footer_for_screen(screen, snapshot),
    }
}

pub fn hud_scene(chrome: ScreenChrome) -> HudScene {
    HudScene::new(
        Element::new(ElementKind::Container, Some(roles::HUD))
            .key(Key::Static("hud"))
            .child(status_bar(&StatusBarProps {
                time: chrome.status.time,
                battery_label: chrome.status.battery_label,
                battery_percent: chrome.status.battery_percent,
                signal_strength: chrome.status.signal_strength,
                network_online: chrome.status.network_online,
            }))
            .child(footer_bar(&FooterBarProps {
                text: chrome.footer_text,
                accent: None,
            })),
    )
}

fn title_for_screen(
    screen: UiScreen,
    snapshot: &RuntimeSnapshot,
    focus_index: usize,
    selected_contact: Option<&ListItemSnapshot>,
) -> String {
    match screen {
        UiScreen::Hub => snapshot
            .hub
            .cards
            .get(focus_index)
            .or_else(|| snapshot.hub.cards.first())
            .map(|card| card.title.clone())
            .unwrap_or_else(|| "Listen".to_string()),
        UiScreen::Listen => "Listen".to_string(),
        UiScreen::Playlists => "Playlists".to_string(),
        UiScreen::RecentTracks => "Recent".to_string(),
        UiScreen::NowPlaying => snapshot.music.title.clone(),
        UiScreen::Ask => snapshot.voice.headline.clone(),
        UiScreen::Talk => "Talk".to_string(),
        UiScreen::Contacts => "More People".to_string(),
        UiScreen::CallHistory => "Recents".to_string(),
        UiScreen::TalkContact => talk_contact_title(snapshot, focus_index, selected_contact),
        UiScreen::VoiceNote => voice_note_title(snapshot, focus_index),
        UiScreen::IncomingCall | UiScreen::OutgoingCall | UiScreen::InCall => {
            call_peer_name(snapshot)
        }
        UiScreen::Power => power_title(snapshot, focus_index),
        UiScreen::Loading => "Loading".to_string(),
        UiScreen::Error => "Error".to_string(),
    }
}

fn footer_for_screen(screen: UiScreen, snapshot: &RuntimeSnapshot) -> String {
    match screen {
        UiScreen::Hub => "Tap = Next | 2x Tap = Open",
        UiScreen::Listen | UiScreen::Contacts => "Tap = Next | 2x Tap = Open | Hold = Back",
        UiScreen::Playlists | UiScreen::RecentTracks => "Tap = Next | 2x Tap = Play | Hold = Back",
        UiScreen::NowPlaying => "Tap = Next | 2x Tap = Play/Pause | Hold = Back",
        UiScreen::Ask => "2x Tap = Ask | Hold = Back",
        UiScreen::Talk => "Tap = Next | 2x Tap = Open | Hold = Back",
        UiScreen::CallHistory => "Tap = Next | 2x Tap = Call | Hold = Back",
        UiScreen::TalkContact => "Tap Next | 2x Select | Hold Back",
        UiScreen::VoiceNote => voice_note_footer(snapshot),
        UiScreen::IncomingCall => "Tap = Answer | Hold = Decline",
        UiScreen::OutgoingCall => "Hold = Cancel",
        UiScreen::InCall => {
            if snapshot.call.muted {
                "Tap = Unmute | Hold = End"
            } else {
                "Tap = Mute | Hold = End"
            }
        }
        UiScreen::Power => "Tap page / Hold back",
        UiScreen::Loading => "",
        UiScreen::Error => "Hold = Back",
    }
    .to_string()
}

fn status_from_snapshot(snapshot: &RuntimeSnapshot) -> HudStatus {
    let battery_percent = snapshot.power.battery_percent.clamp(0, 100) as u8;
    HudStatus {
        time: "00:00".to_string(),
        battery_label: format!("{battery_percent}%"),
        battery_percent,
        signal_strength: signal_strength(snapshot.network.signal_strength),
        network_online: snapshot.network.connected,
    }
}

fn signal_strength(value: i32) -> u8 {
    value.clamp(0, 4) as u8
}

fn talk_contact_title(
    snapshot: &RuntimeSnapshot,
    focus_index: usize,
    selected_contact: Option<&ListItemSnapshot>,
) -> String {
    let has_latest_note = talk_contact_has_latest_note(
        snapshot,
        selected_contact.or_else(|| snapshot.call.contacts.first()),
    );
    match focus_index {
        0 => "Call",
        1 => "Voice Note",
        2 if has_latest_note => "Play Note",
        _ if has_latest_note => "Play Note",
        _ => "Voice Note",
    }
    .to_string()
}

fn talk_contact_has_latest_note(
    snapshot: &RuntimeSnapshot,
    selected_contact: Option<&ListItemSnapshot>,
) -> bool {
    selected_contact
        .and_then(|contact| snapshot.call.latest_voice_note_by_contact.get(&contact.id))
        .is_some_and(|note| !note.local_file_path.trim().is_empty())
}

fn voice_note_title(snapshot: &RuntimeSnapshot, focus_index: usize) -> String {
    let titles: &[&str] = match voice_note_phase(snapshot).as_str() {
        "review" => &["Send", "Play", "Again"],
        "failed" => &["Retry", "Again"],
        "sending" => &["Sending"],
        "sent" => &["Sent"],
        "recording" => &["Recording"],
        _ => &["Voice Note"],
    };
    let selected_index = focus_index.min(titles.len().saturating_sub(1));
    titles
        .get(selected_index)
        .copied()
        .unwrap_or("Voice Note")
        .to_string()
}

fn voice_note_footer(snapshot: &RuntimeSnapshot) -> &'static str {
    match voice_note_phase(snapshot).as_str() {
        "review" | "failed" => "Tap next / Double choose",
        "sending" => "Please wait",
        "sent" => "Double done / Hold back",
        "recording" => "Release to stop",
        _ => "Hold record / Double back",
    }
}

fn voice_note_phase(snapshot: &RuntimeSnapshot) -> String {
    let phase = snapshot.voice.phase.trim().to_ascii_lowercase();
    if snapshot.voice.capture_in_flight || snapshot.voice.ptt_active || phase == "recording" {
        return "recording".to_string();
    }
    if matches!(phase.as_str(), "review" | "sending" | "sent" | "failed") {
        return phase;
    }
    "ready".to_string()
}

fn call_peer_name(snapshot: &RuntimeSnapshot) -> String {
    if snapshot.call.peer_name.trim().is_empty() {
        "Unknown".to_string()
    } else {
        snapshot.call.peer_name.clone()
    }
}

fn power_title(snapshot: &RuntimeSnapshot, focus_index: usize) -> String {
    if !snapshot.power.pages.is_empty() {
        let page = &snapshot.power.pages[focus_index % snapshot.power.pages.len()];
        if page.title.trim().is_empty() {
            "Setup".to_string()
        } else {
            page.title.clone()
        }
    } else if !snapshot.power.rows.is_empty() {
        "Power".to_string()
    } else {
        const DEFAULT_PAGES: [&str; 4] = ["Power", "Time", "Care", "Voice"];
        DEFAULT_PAGES[focus_index % DEFAULT_PAGES.len()].to_string()
    }
}
