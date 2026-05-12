use yoyopod_protocol::ui::{PowerRuntimeSnapshot, RuntimeSnapshot};

pub fn apply(current: &mut RuntimeSnapshot, snapshot: PowerRuntimeSnapshot) {
    current.power = snapshot;
}
