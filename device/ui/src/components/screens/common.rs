use yoyopod_protocol::ui::{ListItemSnapshot, UiScreen};

use crate::animation::presets;
use crate::engine::Key;
use crate::router::FocusPolicy;
use crate::scene::{
    Backdrop, Cursor, Deck, DeckItem, DeckItemAnim, DeckKind, FxLayer, ItemRender, PageModel,
    RegionId, RowModel, Scene, SceneId, Stage,
};

pub fn hero_scene(screen: UiScreen, accent: u32, item_count: usize, focus: usize) -> Scene {
    Scene {
        id: SceneId::new(screen),
        backdrop: Backdrop::AccentDrift {
            accent,
            speed_ms: 800,
        },
        stage: Stage::CenteredHeroIcon,
        decks: vec![Deck {
            kind: DeckKind::CardRow,
            region: RegionId::HeroIcon,
            items: Vec::new(),
            focus_index: focus,
            focus_policy: FocusPolicy::Wrap,
            item_anim: DeckItemAnim::ScaleOnFocus {
                from_permille: 960,
                to_permille: 1000,
            },
            swap_anim: None,
            recycle_window: Some(3),
        }],
        cursor: Some(Cursor::UnderlineDots {
            count: item_count,
            focus,
        }),
        fx: FxLayer::default(),
        modal: None,
        timelines: vec![presets::scene_enter()],
    }
}

pub fn list_scene(
    screen: UiScreen,
    items: &[ListItemSnapshot],
    focus: usize,
    focus_policy: FocusPolicy,
) -> Scene {
    let rows = items.iter().map(row_model).collect::<Vec<_>>();
    Scene {
        id: SceneId::new(screen),
        backdrop: Backdrop::Solid(0x2a2d35),
        stage: Stage::ListWithChrome,
        decks: vec![Deck {
            kind: DeckKind::List,
            region: RegionId::ListBody,
            items: rows
                .into_iter()
                .map(|row| DeckItem {
                    key: Key::String(row.id.clone()),
                    render: ItemRender::Row(row),
                })
                .collect(),
            focus_index: focus,
            focus_policy,
            item_anim: DeckItemAnim::StaggerEnter {
                delay_per_index_ms: 40,
            },
            swap_anim: None,
            recycle_window: Some(4),
        }],
        cursor: Some(Cursor::RowGlow),
        fx: FxLayer::default(),
        modal: None,
        timelines: vec![presets::scene_enter()],
    }
}

pub fn action_scene(screen: UiScreen, focus: usize) -> Scene {
    Scene {
        id: SceneId::new(screen),
        backdrop: Backdrop::Solid(0x2a2d35),
        stage: Stage::TalkActionsGrid,
        decks: vec![Deck {
            kind: DeckKind::Buttons,
            region: RegionId::ButtonRow,
            items: Vec::new(),
            focus_index: focus,
            focus_policy: FocusPolicy::Wrap,
            item_anim: DeckItemAnim::ScaleOnFocus {
                from_permille: 960,
                to_permille: 1000,
            },
            swap_anim: None,
            recycle_window: None,
        }],
        cursor: Some(Cursor::UnderlineDots { count: 3, focus }),
        fx: FxLayer::default(),
        modal: None,
        timelines: vec![presets::scene_enter()],
    }
}

pub fn call_scene(screen: UiScreen, title: String, body: String) -> Scene {
    Scene {
        id: SceneId::new(screen),
        backdrop: Backdrop::Solid(0x2a2d35),
        stage: Stage::CallPanel,
        decks: vec![Deck {
            kind: DeckKind::Page,
            region: RegionId::ListBody,
            items: vec![DeckItem {
                key: Key::Static("call"),
                render: ItemRender::Page(PageModel { title, body }),
            }],
            focus_index: 0,
            focus_policy: FocusPolicy::None,
            item_anim: DeckItemAnim::None,
            swap_anim: None,
            recycle_window: None,
        }],
        cursor: None,
        fx: FxLayer::default(),
        modal: None,
        timelines: vec![presets::scene_enter()],
    }
}

pub fn overlay_scene(screen: UiScreen, modal: crate::scene::Modal) -> Scene {
    Scene {
        id: SceneId::new(screen),
        backdrop: Backdrop::Solid(0x2a2d35),
        stage: Stage::OverlayCenter,
        decks: Vec::new(),
        cursor: None,
        fx: FxLayer::default(),
        modal: Some(modal),
        timelines: vec![presets::scene_enter()],
    }
}

fn row_model(item: &ListItemSnapshot) -> RowModel {
    RowModel {
        id: item.id.clone(),
        title: item.title.clone(),
        subtitle: item.subtitle.clone(),
        icon_key: item.icon_key.clone(),
        selected: false,
    }
}
