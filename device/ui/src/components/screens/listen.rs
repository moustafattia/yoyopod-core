use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::{
    defaults_for, Cursor, Deck, DeckItem, DeckItemAnim, DeckKind, ItemRender, RegionId, RowModel,
    Scene, SceneId,
};

pub struct ListenProps {
    pub rows: Vec<RowModel>,
    pub focus: usize,
}

pub fn props_from(snapshot: &RuntimeSnapshot, focus: usize) -> ListenProps {
    ListenProps {
        rows: items(snapshot)
            .iter()
            .map(|item| RowModel {
                id: item.id.clone(),
                title: item.title.clone(),
                subtitle: item.subtitle.clone(),
                icon_key: item.icon_key.clone(),
                selected: false,
            })
            .collect(),
        focus,
    }
}

pub fn items(_snapshot: &RuntimeSnapshot) -> Vec<ListItemSnapshot> {
    vec![
        ListItemSnapshot::new("playlists", "Playlists", "Saved mixes", "playlist"),
        ListItemSnapshot::new("recent_tracks", "Recent", "Recently played", "recent"),
        ListItemSnapshot::new("shuffle", "Shuffle All", "Start music", "shuffle"),
    ]
}

pub fn scene(props: &ListenProps) -> Scene {
    let defaults = defaults_for(UiScreen::Listen);
    Scene {
        id: SceneId::new(UiScreen::Listen),
        backdrop: defaults.backdrop(0x3ddd53),
        stage: defaults.stage,
        decks: vec![Deck {
            kind: DeckKind::List,
            region: RegionId::ListBody,
            items: props
                .rows
                .iter()
                .map(|row| DeckItem {
                    key: crate::engine::Key::String(row.id.clone()),
                    render: ItemRender::Row(row.clone()),
                })
                .collect(),
            focus_index: props.focus,
            focus_policy: FocusPolicy::Wrap,
            item_anim: DeckItemAnim::StaggerEnter {
                delay_per_index_ms: 40,
            },
            swap_anim: None,
            recycle_window: Some(4),
        }],
        cursor: Some(Cursor::RowGlow),
        fx: defaults.fx_layer(0x3ddd53),
        modal: None,
        timelines: defaults.fx_timelines(),
    }
}
