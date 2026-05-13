use crate::scene::{Backdrop, Cursor, Deck, HudScene, Modal, Scene, SceneGraph};

use super::{Element, ElementKind, Key};

pub fn flatten(graph: &SceneGraph) -> Element {
    Element::new(ElementKind::Container, Some("scene_graph"))
        .key(Key::Static("scene_graph"))
        .child(scene_element(&graph.active))
        .child(hud_element(&graph.hud))
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

fn stage_element(_stage: crate::scene::Stage) -> Element {
    Element::new(ElementKind::Container, Some("scene_stage")).key(Key::Static("stage"))
}

fn decks_element(decks: &[Deck]) -> Element {
    decks.iter().enumerate().fold(
        Element::new(ElementKind::Container, Some("scene_decks")).key(Key::Static("decks")),
        |element, (index, deck)| element.child(deck.element(index)),
    )
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
        .child(
            Element::new(ElementKind::Container, Some("status_bar"))
                .key(Key::Static("status_bar"))
                .child(
                    Element::new(ElementKind::Label, Some("status_signal"))
                        .key(Key::Static("status_signal"))
                        .text(hud.status.signal_strength.to_string()),
                )
                .child(
                    Element::new(ElementKind::Label, Some("status_network"))
                        .key(Key::Static("status_network"))
                        .selected(hud.status.network_online),
                )
                .child(
                    Element::new(ElementKind::Label, Some("status_time"))
                        .key(Key::Static("status_time"))
                        .text(&hud.status.time),
                )
                .child(
                    Element::new(ElementKind::Label, Some("status_battery_label"))
                        .key(Key::Static("status_battery_label"))
                        .text(&hud.status.battery_label),
                ),
        )
        .child(
            Element::new(ElementKind::Container, Some("footer_bar"))
                .key(Key::Static("footer_bar"))
                .child(
                    Element::new(ElementKind::Label, Some("footer_label"))
                        .key(Key::Static("footer_label"))
                        .text(&hud.footer_text),
                ),
        )
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
