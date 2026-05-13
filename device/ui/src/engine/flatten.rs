use crate::animation::{presets, TimelineRef, TrackIndex};
use crate::render_contract::ElementKind;
use crate::scene::{Deck, HudScene, Modal, Scene, SceneGraph};

use super::{AnimSlot, Element, Key};

pub fn flatten(graph: &SceneGraph) -> Element {
    Element::new(ElementKind::Container, Some("scene_graph"))
        .key(Key::Static("scene_graph"))
        .child(scene_element(&graph.active))
        .child(hud_element(&graph.hud))
        .child(modal_stack_element(&graph.modal_stack))
}

pub fn scene_element(scene: &Scene) -> Element {
    let mut root = Element::new(ElementKind::Container, Some("scene_root"))
        .key(Key::String(format!("scene:{}", scene.id.screen.as_str())))
        .with_anim(AnimSlot {
            timeline: TimelineRef(presets::SCENE_ENTER_TIMELINE_ID),
            track: TrackIndex(0),
        });
    root = root.child(scene.backdrop.element());
    root = root.child(stage_element(scene.stage));
    root = root.child(decks_element(&scene.decks));
    if let Some(cursor) = &scene.cursor {
        root = root.child(cursor.element());
    }
    if let Some(fx) = scene.fx.element() {
        root = root.child(fx);
    }
    root
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

fn hud_element(hud: &HudScene) -> Element {
    hud.element()
}

fn modal_stack_element(modal_stack: &[Modal]) -> Element {
    modal_stack.iter().enumerate().fold(
        Element::new(ElementKind::Container, Some("modal_stack")).key(Key::Static("modal_stack")),
        |element, (index, modal)| element.child(modal.element(index)),
    )
}
