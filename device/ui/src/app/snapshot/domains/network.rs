use yoyopod_protocol::ui::{NetworkRuntimeSnapshot, RuntimeSnapshot};

pub fn apply(current: &mut RuntimeSnapshot, snapshot: NetworkRuntimeSnapshot) {
    current.network = snapshot;
}
