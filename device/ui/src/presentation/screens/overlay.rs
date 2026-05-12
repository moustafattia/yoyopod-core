use crate::app::{UiScreen, UiView};
use crate::presentation::screens::{chrome, OverlayViewModel};
use yoyopod_protocol::ui::RuntimeSnapshot;

pub fn loading_model(snapshot: &RuntimeSnapshot) -> OverlayViewModel {
    OverlayViewModel {
        chrome: chrome::chrome(snapshot, ""),
        title: "Loading".to_string(),
        subtitle: snapshot.overlay.message.clone(),
    }
}

pub fn error_model(snapshot: &RuntimeSnapshot) -> OverlayViewModel {
    OverlayViewModel {
        chrome: chrome::chrome(snapshot, "Hold = Back"),
        title: "Error".to_string(),
        subtitle: snapshot.overlay.error.clone(),
    }
}

pub fn loading_view(snapshot: &RuntimeSnapshot) -> UiView {
    UiView {
        screen: UiScreen::Loading,
        title: "Loading".to_string(),
        subtitle: snapshot.overlay.message.clone(),
        footer: String::new(),
        items: Vec::new(),
        focus_index: 0,
    }
}

pub fn error_view(snapshot: &RuntimeSnapshot) -> UiView {
    UiView {
        screen: UiScreen::Error,
        title: "Error".to_string(),
        subtitle: snapshot.overlay.error.clone(),
        footer: "Hold = Back".to_string(),
        items: Vec::new(),
        focus_index: 0,
    }
}
