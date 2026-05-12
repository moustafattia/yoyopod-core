use yoyopod_protocol::ui::{HubRuntimeSnapshot, RuntimeSnapshot};

pub fn apply(current: &mut RuntimeSnapshot, snapshot: HubRuntimeSnapshot) {
    current.hub = snapshot;
}
