use crate::animation::{presets, ActorRef, Timeline, TimelineRef, TrackIndex};
use crate::engine::{AnimSlot, Element, Key};
use crate::render_contract::ElementKind;
use crate::roles;
use crate::router::FocusPolicy;

use super::RegionId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck {
    pub kind: DeckKind,
    pub region: RegionId,
    pub items: Vec<DeckItem>,
    pub focus_index: usize,
    pub focus_policy: FocusPolicy,
    pub item_anim: DeckItemAnim,
    pub swap_anim: Option<crate::animation::Transition>,
    pub recycle_window: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckKind {
    CardRow,
    List,
    Page,
    Grid,
    Buttons,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeckItem {
    pub key: Key,
    pub render: ItemRender,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemRender {
    Card(CardModel),
    Row(RowModel),
    Page(PageModel),
    Button(ButtonModel),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardModel {
    pub title: String,
    pub subtitle: String,
    pub icon_key: String,
    pub accent: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowModel {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon_key: String,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageModel {
    pub title: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ButtonModel {
    pub title: String,
    pub icon_key: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeckItemAnim {
    None,
    ScaleOnFocus {
        from_permille: u16,
        to_permille: u16,
    },
    BreatheWhenFocused,
    StaggerEnter {
        delay_per_index_ms: u32,
    },
}

impl Deck {
    pub fn element(&self, index: usize) -> Element {
        let mut element = Element::new(ElementKind::Container, Some(deck_role(self.kind)))
            .key(Key::Indexed(index))
            .child(
                Element::new(ElementKind::Container, Some(roles::DECK_REGION))
                    .key(Key::Static("deck_region"))
                    .region(self.region),
            );
        for (visible_index, (item_index, item)) in self.visible_items().enumerate() {
            element = element.child(deck_item_element(
                item,
                item_index == self.focus_index,
                self.item_anim,
                index,
                visible_index,
            ));
        }
        element
    }

    fn visible_items(&self) -> impl Iterator<Item = (usize, &DeckItem)> {
        let range = self.visible_range();
        self.items
            .iter()
            .enumerate()
            .skip(range.start)
            .take(range.end.saturating_sub(range.start))
    }

    fn visible_range(&self) -> std::ops::Range<usize> {
        let len = self.items.len();
        if len == 0 {
            return 0..0;
        }

        let focus = self.focus_index.min(len.saturating_sub(1));
        let window = match self.kind {
            DeckKind::Page => 1,
            _ => self.recycle_window.unwrap_or(len),
        }
        .clamp(1, len);

        let mut start = focus.saturating_sub(window / 2);
        if start + window > len {
            start = len - window;
        }
        start..start + window
    }

    pub fn focused_visible_index(&self) -> usize {
        let range = self.visible_range();
        self.focus_index
            .min(self.items.len().saturating_sub(1))
            .saturating_sub(range.start)
    }

    pub fn enter_timeline(&self) -> Option<Timeline> {
        match self.item_anim {
            DeckItemAnim::StaggerEnter { delay_per_index_ms } => {
                Some(presets::stagger_enter(delay_per_index_ms))
            }
            DeckItemAnim::None
            | DeckItemAnim::ScaleOnFocus { .. }
            | DeckItemAnim::BreatheWhenFocused => None,
        }
    }

    pub fn item_timelines(&self, deck_index: usize) -> Vec<Timeline> {
        match self.item_anim {
            DeckItemAnim::BreatheWhenFocused if !self.items.is_empty() => {
                vec![presets::breathe_focused_item(
                    deck_index,
                    self.focused_visible_index(),
                )]
            }
            DeckItemAnim::None
            | DeckItemAnim::ScaleOnFocus { .. }
            | DeckItemAnim::StaggerEnter { .. }
            | DeckItemAnim::BreatheWhenFocused => Vec::new(),
        }
    }
}

fn deck_item_element(
    item: &DeckItem,
    selected: bool,
    item_anim: DeckItemAnim,
    deck_index: usize,
    visible_index: usize,
) -> Element {
    let element = match &item.render {
        ItemRender::Card(card) => Element::new(ElementKind::Container, Some(roles::CARD))
            .key(item.key.clone())
            .accent(card.accent)
            .child(Element::new(ElementKind::Label, Some(roles::CARD_TITLE)).text(&card.title))
            .child(
                Element::new(ElementKind::Label, Some(roles::CARD_SUBTITLE)).text(&card.subtitle),
            )
            .child(Element::new(ElementKind::Image, Some(roles::CARD_ICON)).icon(&card.icon_key)),
        ItemRender::Row(row) => Element::new(ElementKind::Container, Some(roles::LIST_ROW))
            .key(item.key.clone())
            .selected(row.selected || selected)
            .child(Element::new(ElementKind::Image, Some(roles::LIST_ROW_ICON)).icon(&row.icon_key))
            .child(Element::new(ElementKind::Label, Some(roles::LIST_ROW_TITLE)).text(&row.title))
            .child(
                Element::new(ElementKind::Label, Some(roles::LIST_ROW_SUBTITLE))
                    .text(&row.subtitle),
            ),
        ItemRender::Page(page) => Element::new(ElementKind::Container, Some(roles::PAGE))
            .key(item.key.clone())
            .child(Element::new(ElementKind::Label, Some(roles::PAGE_TITLE)).text(&page.title))
            .child(Element::new(ElementKind::Label, Some(roles::PAGE_BODY)).text(&page.body)),
        ItemRender::Button(button) => Element::new(ElementKind::Container, Some(roles::BUTTON))
            .key(item.key.clone())
            .child(
                Element::new(ElementKind::Image, Some(roles::BUTTON_ICON)).icon(&button.icon_key),
            )
            .child(Element::new(ElementKind::Label, Some(roles::BUTTON_TITLE)).text(&button.title)),
    }
    .actor(ActorRef::DeckItem {
        deck: deck_index,
        index: visible_index,
    });
    match item_anim {
        DeckItemAnim::StaggerEnter { .. } => element.with_anim(AnimSlot {
            timeline: TimelineRef(presets::STAGGER_ENTER_TIMELINE_ID),
            track: TrackIndex(visible_index.min(3)),
        }),
        DeckItemAnim::ScaleOnFocus {
            from_permille,
            to_permille,
        } => element.scale_permille(if selected {
            i32::from(to_permille)
        } else {
            i32::from(from_permille)
        }),
        DeckItemAnim::BreatheWhenFocused => {
            element.scale_permille(if selected { 1000 } else { 960 })
        }
        DeckItemAnim::None => element,
    }
}

const fn deck_role(kind: DeckKind) -> &'static str {
    match kind {
        DeckKind::CardRow => roles::DECK_CARD_ROW,
        DeckKind::List => roles::DECK_LIST,
        DeckKind::Page => roles::DECK_PAGE,
        DeckKind::Grid => roles::DECK_GRID,
        DeckKind::Buttons => roles::DECK_BUTTONS,
    }
}
