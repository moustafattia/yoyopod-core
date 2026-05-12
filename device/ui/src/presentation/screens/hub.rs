use crate::app::UiView;
use crate::presentation::screens::{chrome, HubCardModel, HubViewModel};
use yoyopod_protocol::ui::UiScreen;
use yoyopod_protocol::ui::{ListItemSnapshot, RuntimeSnapshot};

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

pub fn view(snapshot: &RuntimeSnapshot, focus_index: usize) -> UiView {
    let cards = snapshot.hub.cards.clone();
    let focused = cards.get(focus_index).or_else(|| cards.first());

    UiView {
        screen: UiScreen::Hub,
        title: focused
            .map(|card| card.title.clone())
            .unwrap_or_else(|| "Listen".to_string()),
        subtitle: focused
            .map(|card| card.subtitle.clone())
            .unwrap_or_default(),
        footer: "Tap = Next | 2x Tap = Open".to_string(),
        items: cards
            .iter()
            .map(|card| {
                ListItemSnapshot::new(
                    card.key.clone(),
                    card.title.clone(),
                    card.subtitle.clone(),
                    card.key.clone(),
                )
            })
            .collect(),
        focus_index,
    }
}
