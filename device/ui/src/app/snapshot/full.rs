use yoyopod_protocol::ui::{RuntimeSnapshot, RuntimeSnapshotDomain};

use super::SnapshotChange;

pub fn replace_full(current: &mut RuntimeSnapshot, snapshot: RuntimeSnapshot) -> SnapshotChange {
    let previous_app_state = current.app_state;
    let app_state = snapshot.app_state;
    *current = snapshot;
    SnapshotChange {
        domain: RuntimeSnapshotDomain::Full,
        previous_app_state,
        app_state,
    }
}
