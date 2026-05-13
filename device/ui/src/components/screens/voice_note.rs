use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::engine::Key;
use crate::scene::{ButtonModel, DeckItem, ItemRender, Scene, SceneDefaults};

pub struct VoiceNoteProps {
    pub buttons: Vec<DeckItem>,
    pub focus: usize,
}

pub fn props_from(snapshot: &RuntimeSnapshot, focus: usize) -> VoiceNoteProps {
    VoiceNoteProps {
        buttons: buttons(snapshot),
        focus,
    }
}

pub fn scene(props: &VoiceNoteProps, defaults: &SceneDefaults) -> Scene {
    let mut scene = super::common::action_scene(UiScreen::VoiceNote, defaults, props.focus);
    if let Some(deck) = scene.decks.first_mut() {
        deck.items = props.buttons.clone();
    }
    scene.cursor = Some(crate::scene::Cursor::UnderlineDots {
        count: scene
            .decks
            .first()
            .map(|deck| deck.items.len())
            .unwrap_or(0),
        focus: props.focus,
    });
    scene
}

fn buttons(snapshot: &RuntimeSnapshot) -> Vec<DeckItem> {
    match voice_note_phase(snapshot).as_str() {
        "review" => vec![
            button("send", "Send", "check"),
            button("play", "Play", "play"),
            button("again", "Again", "close"),
        ],
        "failed" => vec![
            button("retry", "Retry", "retry"),
            button("again", "Again", "close"),
        ],
        "sending" => vec![button("sending", "Sending", "voice_note")],
        "sent" => vec![button("sent", "Sent", "check")],
        "recording" => vec![button("recording", "Recording", "voice_note")],
        _ => vec![button("record", "Voice Note", "voice_note")],
    }
}

fn button(key: &'static str, title: &'static str, icon_key: &'static str) -> DeckItem {
    DeckItem {
        key: Key::Static(key),
        render: ItemRender::Button(ButtonModel {
            title: title.to_string(),
            icon_key: icon_key.to_string(),
        }),
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
