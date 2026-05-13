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
