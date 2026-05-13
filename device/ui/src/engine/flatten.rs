use crate::scene::{
    Backdrop, Cursor, Deck, DeckItem, DeckKind, HudScene, ItemRender, Modal, Scene, SceneGraph,
};

use super::{Element, ElementKind, Key};

pub fn flatten(graph: &SceneGraph) -> Element {
    Element::new(ElementKind::Container, Some("scene_graph"))
        .key(Key::Static("scene_graph"))
        .child(hud_element(&graph.hud))
        .child(scene_element(&graph.active))
        .child(modal_stack_element(&graph.modal_stack))
}

pub fn scene_element(scene: &Scene) -> Element {
    let mut root = Element::new(ElementKind::Container, Some("scene_root"))
        .key(Key::String(format!("scene:{}", scene.id.screen.as_str())));
    root = root.child(backdrop_element(scene.backdrop));
    root = root.child(stage_element(scene.stage));
    root = root.child(decks_element(&scene.decks));
    if let Some(cursor) = &scene.cursor {
        root = root.child(cursor_element(cursor));
    }
    root
}

fn backdrop_element(backdrop: Backdrop) -> Element {
    let mut element =
        Element::new(ElementKind::Container, Some("scene_backdrop")).key(Key::Static("backdrop"));
    element.props.variant = Some(match backdrop {
        Backdrop::Solid(_) => "solid",
        Backdrop::Gradient { .. } => "gradient",
        Backdrop::AccentDrift { .. } => "accent_drift",
        Backdrop::Vignette { .. } => "vignette",
    });
    element
}

fn stage_element(stage: crate::scene::Stage) -> Element {
    Element::new(ElementKind::Container, Some("scene_stage"))
        .key(Key::Static("stage"))
        .text(format!("{stage:?}"))
}

fn decks_element(decks: &[Deck]) -> Element {
    decks.iter().enumerate().fold(
        Element::new(ElementKind::Container, Some("scene_decks")).key(Key::Static("decks")),
        |element, (index, deck)| element.child(deck_element(index, deck)),
    )
}

fn deck_element(index: usize, deck: &Deck) -> Element {
    let mut element = Element::new(ElementKind::Container, Some(deck_role(deck.kind)))
        .key(Key::Indexed(index))
        .selected(!deck.items.is_empty())
        .child(
            Element::new(ElementKind::Container, Some("deck_region"))
                .text(format!("{:?}", deck.region)),
        );
    for item in &deck.items {
        element = element.child(deck_item_element(item));
    }
    element
}

fn deck_item_element(item: &DeckItem) -> Element {
    match &item.render {
        ItemRender::Card(card) => Element::new(ElementKind::Container, Some("card"))
            .key(item.key.clone())
            .accent(card.accent)
            .child(Element::new(ElementKind::Label, Some("card_title")).text(&card.title))
            .child(Element::new(ElementKind::Label, Some("card_subtitle")).text(&card.subtitle))
            .child(Element::new(ElementKind::Image, Some("card_icon")).icon(&card.icon_key)),
        ItemRender::Row(row) => Element::new(ElementKind::Container, Some("list_row"))
            .key(item.key.clone())
            .selected(row.selected)
            .child(Element::new(ElementKind::Image, Some("list_row_icon")).icon(&row.icon_key))
            .child(Element::new(ElementKind::Label, Some("list_row_title")).text(&row.title))
            .child(Element::new(ElementKind::Label, Some("list_row_subtitle")).text(&row.subtitle)),
        ItemRender::Page(page) => Element::new(ElementKind::Container, Some("page"))
            .key(item.key.clone())
            .child(Element::new(ElementKind::Label, Some("page_title")).text(&page.title))
            .child(Element::new(ElementKind::Label, Some("page_body")).text(&page.body)),
        ItemRender::Button(button) => Element::new(ElementKind::Container, Some("button"))
            .key(item.key.clone())
            .child(Element::new(ElementKind::Image, Some("button_icon")).icon(&button.icon_key))
            .child(Element::new(ElementKind::Label, Some("button_title")).text(&button.title)),
    }
}

fn cursor_element(cursor: &Cursor) -> Element {
    match cursor {
        Cursor::UnderlineDots { count, focus } => {
            Element::new(ElementKind::Container, Some("cursor_dots"))
                .key(Key::Static("cursor"))
                .text(format!("{focus}/{count}"))
        }
        Cursor::RowGlow => {
            Element::new(ElementKind::Container, Some("cursor_row_glow")).key(Key::Static("cursor"))
        }
    }
}

fn hud_element(hud: &HudScene) -> Element {
    Element::new(ElementKind::Container, Some("hud"))
        .key(Key::Static("hud"))
        .child(Element::new(ElementKind::Label, Some("status_bar")).text(&hud.status_text))
        .child(Element::new(ElementKind::Label, Some("footer_bar")).text(&hud.footer_text))
}

fn modal_stack_element(modal_stack: &[Modal]) -> Element {
    modal_stack.iter().enumerate().fold(
        Element::new(ElementKind::Container, Some("modal_stack")).key(Key::Static("modal_stack")),
        |element, (index, modal)| element.child(modal_element(index, modal)),
    )
}

fn modal_element(index: usize, modal: &Modal) -> Element {
    match modal {
        Modal::Loading { title, message } => modal_content(index, "loading", title, message),
        Modal::Error { title, message } => modal_content(index, "error", title, message),
    }
}

fn modal_content(index: usize, variant: &'static str, title: &str, message: &str) -> Element {
    let mut element = Element::new(ElementKind::Container, Some("modal")).key(Key::Indexed(index));
    element.props.variant = Some(variant);
    element
        .child(Element::new(ElementKind::Label, Some("modal_title")).text(title))
        .child(Element::new(ElementKind::Label, Some("modal_message")).text(message))
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
