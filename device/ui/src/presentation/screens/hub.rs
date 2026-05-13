use crate::presentation::screens::{chrome, HubCardModel, HubViewModel};
use yoyopod_protocol::ui::RuntimeSnapshot;

pub fn model(snapshot: &RuntimeSnapshot, focus_index: usize) -> HubViewModel {
    HubViewModel {
        chrome: chrome::chrome(snapshot, "Tap = Next | 2x Tap = Open"),
        cards: snapshot
            .hub
            .cards
            .iter()
            .map(|card| HubCardModel {
                key: card.key.clone(),
                title: card.title.clone(),
                subtitle: card.subtitle.clone(),
                accent: card.accent,
            })
            .collect(),
        selected_index: focus_index,
    }
}

pub fn focused_card(model: &HubViewModel) -> Option<&HubCardModel> {
    model
        .cards
        .get(model.selected_index)
        .or_else(|| model.cards.first())
}

pub fn focused_title(model: &HubViewModel) -> &str {
    focused_card(model)
        .map(|card| card.title.as_str())
        .unwrap_or("Listen")
}
