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

#[cfg(test)]
mod tests {
    use super::*;
    use yoyopod_protocol::ui::UiScreen;

    #[test]
    fn full_snapshot_replaces_current_snapshot_and_reports_app_state_change() {
        let mut current = RuntimeSnapshot::default();
        current.app_state = UiScreen::Listen;
        let mut next = RuntimeSnapshot::default();
        next.app_state = UiScreen::Talk;

        let change = replace_full(&mut current, next);

        assert_eq!(change.domain, RuntimeSnapshotDomain::Full);
        assert_eq!(change.previous_app_state, UiScreen::Listen);
        assert_eq!(change.app_state, UiScreen::Talk);
        assert_eq!(current.app_state, UiScreen::Talk);
    }
}
