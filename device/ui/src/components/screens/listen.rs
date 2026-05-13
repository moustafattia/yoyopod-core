use yoyopod_protocol::ui::{RuntimeSnapshot, UiScreen};

use crate::router::FocusPolicy;
use crate::scene::{
    Backdrop, Cursor, Deck, DeckItem, DeckItemAnim, DeckKind, ItemRender, RegionId, RowModel,
    Scene, SceneId, Stage,
};

pub struct ListenProps {
    pub rows: Vec<RowModel>,
    pub focus: usize,
}

pub fn props_from(snapshot: &RuntimeSnapshot, focus: usize) -> ListenProps {
    ListenProps {
        rows: snapshot
            .music
            .playlists
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

pub fn scene(props: &ListenProps) -> Scene {
    Scene {
        id: SceneId::new(UiScreen::Listen),
        backdrop: Backdrop::Solid(0x2a2d35),
        stage: Stage::ListWithChrome,
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
        fx: Default::default(),
        modal: None,
        timelines: vec![crate::animation::presets::scene_enter()],
    }
}
