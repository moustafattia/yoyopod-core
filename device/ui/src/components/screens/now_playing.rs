use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::engine::Key;
use crate::scene::{
    defaults_for, Deck, DeckItem, DeckItemAnim, DeckKind, ItemRender, PageModel, RegionId, Scene,
    SceneId,
};

pub struct NowPlayingProps {
    pub title: String,
    pub body: String,
}

pub fn props_from(snapshot: &RuntimeSnapshot) -> NowPlayingProps {
    NowPlayingProps {
        title: snapshot.music.title.clone(),
        body: now_playing_body(snapshot),
    }
}

pub fn scene(props: &NowPlayingProps) -> Scene {
    let defaults = defaults_for(UiScreen::NowPlaying);
    Scene {
        id: SceneId::new(UiScreen::NowPlaying),
        backdrop: defaults.backdrop(0x3ddd53),
        stage: defaults.stage,
        decks: vec![Deck {
            kind: DeckKind::Page,
            region: RegionId::ListBody,
            items: vec![DeckItem {
                key: Key::Static("now_playing"),
                render: ItemRender::Page(PageModel {
                    title: props.title.clone(),
                    body: props.body.clone(),
                }),
            }],
            focus_index: 0,
            focus_policy: crate::router::FocusPolicy::None,
            item_anim: DeckItemAnim::None,
            swap_anim: None,
            recycle_window: None,
        }],
        cursor: None,
        fx: defaults.fx_layer(0x3ddd53),
        modal: None,
        timelines: defaults.fx_timelines(),
    }
}

fn now_playing_body(snapshot: &RuntimeSnapshot) -> String {
    let state = if snapshot.music.playing {
        "Now Playing"
    } else if snapshot.music.paused {
        "Paused"
    } else {
        "Stopped"
    };
    let artist = if snapshot.music.artist.trim().is_empty() {
        "Unknown artist"
    } else {
        snapshot.music.artist.as_str()
    };
    format!(
        "{artist}\n{state}\nProgress: {}%",
        (snapshot.music.progress_permille / 10).clamp(0, 100)
    )
}
