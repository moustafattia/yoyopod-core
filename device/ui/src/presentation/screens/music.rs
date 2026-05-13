use crate::presentation::screens::{chrome, ListScreenModel, NowPlayingViewModel};
use yoyopod_protocol::ui::RuntimeSnapshot;

pub fn playlists_model(snapshot: &RuntimeSnapshot, focus_index: usize) -> ListScreenModel {
    ListScreenModel {
        chrome: chrome::chrome(snapshot, "Tap = Next | 2x Tap = Play | Hold = Back"),
        title: "Playlists".to_string(),
        subtitle: "Saved mixes".to_string(),
        rows: chrome::list_rows(&snapshot.music.playlists, focus_index),
    }
}

pub fn recent_tracks_model(snapshot: &RuntimeSnapshot, focus_index: usize) -> ListScreenModel {
    ListScreenModel {
        chrome: chrome::chrome(snapshot, "Tap = Next | 2x Tap = Play | Hold = Back"),
        title: "Recent".to_string(),
        subtitle: "Recently played".to_string(),
        rows: chrome::list_rows(&snapshot.music.recent_tracks, focus_index),
    }
}

pub fn now_playing_model(snapshot: &RuntimeSnapshot) -> NowPlayingViewModel {
    NowPlayingViewModel {
        chrome: chrome::chrome(snapshot, "Tap = Next | 2x Tap = Play/Pause | Hold = Back"),
        title: snapshot.music.title.clone(),
        artist: snapshot.music.artist.clone(),
        state_text: if snapshot.music.playing {
            "Now Playing".to_string()
        } else if snapshot.music.paused {
            "Paused".to_string()
        } else {
            "Stopped".to_string()
        },
        progress_permille: snapshot.music.progress_permille,
    }
}
