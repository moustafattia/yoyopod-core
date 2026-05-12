use yoyopod_protocol::ui::{RuntimeSnapshot, RuntimeSnapshotDomain, RuntimeSnapshotPatch};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotChange {
    pub domain: RuntimeSnapshotDomain,
    pub previous_app_state: String,
    pub app_state: String,
}

pub fn replace_full(current: &mut RuntimeSnapshot, snapshot: RuntimeSnapshot) -> SnapshotChange {
    let previous_app_state = current.app_state.clone();
    let app_state = snapshot.app_state.clone();
    *current = snapshot;
    SnapshotChange {
        domain: RuntimeSnapshotDomain::Full,
        previous_app_state,
        app_state,
    }
}

pub fn apply_patch(current: &mut RuntimeSnapshot, patch: RuntimeSnapshotPatch) -> SnapshotChange {
    let previous_app_state = current.app_state.clone();
    let domain = patch.domain();

    match patch {
        RuntimeSnapshotPatch::Full(snapshot) => *current = snapshot,
        RuntimeSnapshotPatch::AppState(app_state) => current.app_state = app_state,
        RuntimeSnapshotPatch::Hub(hub) => current.hub = hub,
        RuntimeSnapshotPatch::Music(music) => current.music = music,
        RuntimeSnapshotPatch::Call(call) => current.call = call,
        RuntimeSnapshotPatch::Voice(voice) => current.voice = voice,
        RuntimeSnapshotPatch::Power(power) => current.power = power,
        RuntimeSnapshotPatch::Network(network) => current.network = network,
        RuntimeSnapshotPatch::Overlay(overlay) => current.overlay = overlay,
    }

    SnapshotChange {
        domain,
        previous_app_state,
        app_state: current.app_state.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yoyopod_protocol::ui::MusicRuntimeSnapshot;

    #[test]
    fn full_snapshot_replaces_current_snapshot_and_reports_app_state_change() {
        let mut current = RuntimeSnapshot::default();
        current.app_state = "listen".to_string();
        let mut next = RuntimeSnapshot::default();
        next.app_state = "talk".to_string();

        let change = replace_full(&mut current, next);

        assert_eq!(change.domain, RuntimeSnapshotDomain::Full);
        assert_eq!(change.previous_app_state, "listen");
        assert_eq!(change.app_state, "talk");
        assert_eq!(current.app_state, "talk");
    }

    #[test]
    fn domain_patch_updates_only_that_snapshot_domain() {
        let mut current = RuntimeSnapshot::default();
        current.app_state = "listen".to_string();
        current.music.title = "Before".to_string();

        let change = apply_patch(
            &mut current,
            RuntimeSnapshotPatch::Music(MusicRuntimeSnapshot {
                title: "After".to_string(),
                ..MusicRuntimeSnapshot::default()
            }),
        );

        assert_eq!(change.domain, RuntimeSnapshotDomain::Music);
        assert_eq!(change.previous_app_state, "listen");
        assert_eq!(change.app_state, "listen");
        assert_eq!(current.music.title, "After");
    }

    #[test]
    fn app_state_patch_reports_old_and_new_app_state() {
        let mut current = RuntimeSnapshot::default();
        current.app_state = "listen".to_string();

        let change = apply_patch(
            &mut current,
            RuntimeSnapshotPatch::AppState("power".to_string()),
        );

        assert_eq!(change.domain, RuntimeSnapshotDomain::AppState);
        assert_eq!(change.previous_app_state, "listen");
        assert_eq!(change.app_state, "power");
    }
}
