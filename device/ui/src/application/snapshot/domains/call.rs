use yoyopod_protocol::ui::{CallRuntimeSnapshot, RuntimeSnapshot};

pub fn apply(current: &mut RuntimeSnapshot, snapshot: CallRuntimeSnapshot) {
    current.call = snapshot;
}
