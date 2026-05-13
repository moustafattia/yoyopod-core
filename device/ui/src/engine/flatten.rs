use crate::animation::{presets, TimelineRef, TrackIndex};
use crate::render_contract::ElementKind;
use crate::scene::{Deck, HudScene, LayerSlot, Modal, Scene, SceneGraph, LAYER_ORDER};

use super::{AnimSlot, Element, Key};

pub fn flatten(graph: &SceneGraph) -> Element {
    LAYER_ORDER
        .into_iter()
        .filter(|slot| slot.is_graph_overlay())
        .fold(
            Element::new(ElementKind::Container, Some("scene_graph"))
                .key(Key::Static("scene_graph"))
                .child(scene_element(&graph.active)),
            |element, slot| match graph_overlay_element(graph, slot) {
                Some(layer) => element.child(layer),
                None => element,
            },
        )
}

pub fn scene_element(scene: &Scene) -> Element {
    let root = Element::new(ElementKind::Container, Some("scene_root"))
        .key(Key::String(format!("scene:{}", scene.id.screen.as_str())))
        .with_anim(AnimSlot {
            timeline: TimelineRef(presets::SCENE_ENTER_TIMELINE_ID),
            track: TrackIndex(0),
        });
    LAYER_ORDER
        .into_iter()
        .filter(|slot| slot.is_scene_owned())
        .fold(root, |element, slot| {
            match scene_layer_element(scene, slot) {
                Some(layer) => element.child(layer),
                None => element,
            }
        })
}

fn scene_layer_element(scene: &Scene, slot: LayerSlot) -> Option<Element> {
    match slot {
        LayerSlot::Backdrop => Some(scene.backdrop.element()),
        LayerSlot::Stage => Some(stage_element(scene.stage)),
        LayerSlot::Decks => Some(decks_element(&scene.decks)),
        LayerSlot::Cursor => scene.cursor.as_ref().map(|cursor| cursor.element()),
        LayerSlot::Fx => scene.fx.element(),
        LayerSlot::Hud | LayerSlot::Modal => None,
    }
}

fn graph_overlay_element(graph: &SceneGraph, slot: LayerSlot) -> Option<Element> {
    match slot {
        LayerSlot::Hud => Some(hud_element(&graph.hud)),
        LayerSlot::Modal => Some(modal_stack_element(&graph.modal_stack)),
        LayerSlot::Backdrop
        | LayerSlot::Stage
        | LayerSlot::Decks
        | LayerSlot::Cursor
        | LayerSlot::Fx => None,
    }
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
