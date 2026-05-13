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

use crate::animation::presets;
use crate::router;
use crate::scene::{defaults_for, Scene};

pub fn scene_for_screen(
    screen: UiScreen,
    snapshot: &RuntimeSnapshot,
    focus: usize,
    selected_contact: Option<&ListItemSnapshot>,
) -> Scene {
    let defaults = defaults_for(screen);
    let scene = match screen {
        UiScreen::Hub => hub::scene(&hub::props_from(snapshot, focus), &defaults),
        UiScreen::Listen => listen::scene(&listen::props_from(snapshot, focus), &defaults),
        UiScreen::Playlists => playlists::scene(&playlists::props_from(snapshot, focus), &defaults),
        UiScreen::RecentTracks => {
            recent_tracks::scene(&recent_tracks::props_from(snapshot, focus), &defaults)
        }
        UiScreen::NowPlaying => now_playing::scene(&now_playing::props_from(snapshot), &defaults),
        UiScreen::Ask => ask::scene(&ask::props_from(snapshot, focus), &defaults),
        UiScreen::Talk => talk::scene(&talk::props_from(focus), &defaults),
        UiScreen::Contacts => contacts::scene(&contacts::props_from(snapshot, focus), &defaults),
        UiScreen::CallHistory => {
            call_history::scene(&call_history::props_from(snapshot, focus), &defaults)
        }
        UiScreen::TalkContact => talk_contact::scene(
            &talk_contact::props_from(snapshot, focus, selected_contact),
            &defaults,
        ),
        UiScreen::VoiceNote => {
            voice_note::scene(&voice_note::props_from(snapshot, focus), &defaults)
        }
        UiScreen::IncomingCall => {
            incoming_call::scene(&incoming_call::props_from(snapshot), &defaults)
        }
        UiScreen::OutgoingCall => {
            outgoing_call::scene(&outgoing_call::props_from(snapshot), &defaults)
        }
        UiScreen::InCall => in_call::scene(&in_call::props_from(snapshot), &defaults),
        UiScreen::Power => power::scene(&power::props_from(snapshot, focus), &defaults),
        UiScreen::Loading => loading::scene(&loading::props_from(snapshot), &defaults),
        UiScreen::Error => error::scene(&error::props_from(snapshot), &defaults),
    };
    with_route_timelines(screen, scene)
}

fn with_route_timelines(screen: UiScreen, mut scene: Scene) -> Scene {
    if let Some(timeline) = router::route_for(screen).on_enter {
        let enter_timeline = if timeline.0 == presets::STAGGER_ENTER_TIMELINE_ID {
            scene
                .decks
                .iter()
                .find_map(|deck| deck.enter_timeline())
                .unwrap_or_else(|| presets::timeline_for_ref(timeline))
        } else {
            presets::timeline_for_ref(timeline)
        };
        scene.timelines.insert(0, enter_timeline);
    }
    let item_timelines = scene
        .decks
        .iter()
        .enumerate()
        .flat_map(|(deck_index, deck)| deck.item_timelines(deck_index))
        .collect::<Vec<_>>();
    scene.timelines.extend(item_timelines);
    scene
}
