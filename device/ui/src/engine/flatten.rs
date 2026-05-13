use crate::animation::{presets, ActorRef, TimelineRef, TrackIndex};
use crate::components::widgets::{footer_bar, status_bar, FooterBarProps, StatusBarProps};
use crate::render_contract::ElementKind;
use crate::roles;
use crate::scene::{Deck, HudScene, LayerSlot, Modal, Scene, SceneGraph, LAYER_ORDER};

use super::{AnimSlot, Element, Key};

pub fn flatten(graph: &SceneGraph) -> Element {
    LAYER_ORDER
        .into_iter()
        .filter(|slot| slot.is_graph_overlay())
        .fold(
            Element::new(ElementKind::Container, Some(roles::SCENE_GRAPH))
                .key(Key::Static("scene_graph"))
                .child(scene_element(&graph.active)),
            |element, slot| match graph_overlay_element(graph, slot) {
                Some(layer) => element.child(layer),
                None => element,
            },
        )
}

pub fn scene_element(scene: &Scene) -> Element {
    let root = Element::new(ElementKind::Container, Some(roles::SCENE_ROOT))
        .key(Key::String(format!("scene:{}", scene.id.screen.as_str())))
        .actor(ActorRef::Screen)
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
    Element::new(ElementKind::Container, Some(roles::SCENE_STAGE)).key(Key::Static("stage"))
}

fn decks_element(decks: &[Deck]) -> Element {
    decks.iter().enumerate().fold(
        Element::new(ElementKind::Container, Some(roles::SCENE_DECKS)).key(Key::Static("decks")),
        |element, (index, deck)| element.child(deck.element(index)),
    )
}

fn hud_element(hud: &HudScene) -> Element {
    Element::new(ElementKind::Container, Some(roles::HUD))
        .key(Key::Static("hud"))
        .child(status_bar(&StatusBarProps {
            time: hud.status.time.clone(),
            battery_label: hud.status.battery_label.clone(),
            battery_percent: hud.status.battery_percent,
            signal_strength: hud.status.signal_strength,
            network_online: hud.status.network_online,
        }))
        .child(footer_bar(&FooterBarProps {
            text: hud.footer_text.clone(),
            accent: None,
        }))
}

fn modal_stack_element(modal_stack: &[Modal]) -> Element {
    modal_stack.iter().enumerate().fold(
        Element::new(ElementKind::Container, Some(roles::MODAL_STACK))
            .key(Key::Static("modal_stack")),
        |element, (index, modal)| element.child(modal.element(index)),
    )
}
