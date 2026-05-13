use crate::engine::{Element, ElementKind, Key};
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
            .selected(!self.items.is_empty())
            .child(
                Element::new(ElementKind::Container, Some("deck_region"))
                    .key(Key::Static("deck_region")),
            );
        for (item_index, item) in self.items.iter().enumerate() {
            element = element.child(deck_item_element(item, item_index == self.focus_index));
        }
        element
    }
}

fn deck_item_element(item: &DeckItem, selected: bool) -> Element {
    match &item.render {
        ItemRender::Card(card) => Element::new(ElementKind::Container, Some("card"))
            .key(item.key.clone())
            .selected(selected)
            .accent(card.accent)
            .child(Element::new(ElementKind::Label, Some("card_title")).text(&card.title))
            .child(Element::new(ElementKind::Label, Some("card_subtitle")).text(&card.subtitle))
            .child(Element::new(ElementKind::Image, Some("card_icon")).icon(&card.icon_key)),
        ItemRender::Row(row) => Element::new(ElementKind::Container, Some("list_row"))
            .key(item.key.clone())
            .selected(row.selected || selected)
            .child(Element::new(ElementKind::Image, Some("list_row_icon")).icon(&row.icon_key))
            .child(Element::new(ElementKind::Label, Some("list_row_title")).text(&row.title))
            .child(Element::new(ElementKind::Label, Some("list_row_subtitle")).text(&row.subtitle)),
        ItemRender::Page(page) => Element::new(ElementKind::Container, Some("page"))
            .key(item.key.clone())
            .selected(selected)
            .child(Element::new(ElementKind::Label, Some("page_title")).text(&page.title))
            .child(Element::new(ElementKind::Label, Some("page_body")).text(&page.body)),
        ItemRender::Button(button) => Element::new(ElementKind::Container, Some("button"))
            .key(item.key.clone())
            .selected(selected)
            .child(Element::new(ElementKind::Image, Some("button_icon")).icon(&button.icon_key))
            .child(Element::new(ElementKind::Label, Some("button_title")).text(&button.title)),
    }
}

const fn deck_role(kind: DeckKind) -> &'static str {
    match kind {
        DeckKind::CardRow => "deck_card_row",
        DeckKind::List => "deck_list",
        DeckKind::Page => "deck_page",
        DeckKind::Grid => "deck_grid",
        DeckKind::Buttons => "deck_buttons",
    }
}
