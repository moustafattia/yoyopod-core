use yoyopod_protocol::ui::{OverlayRuntimeSnapshot, RuntimeSnapshot};

pub fn apply(current: &mut RuntimeSnapshot, snapshot: OverlayRuntimeSnapshot) {
    current.overlay = snapshot;
}
