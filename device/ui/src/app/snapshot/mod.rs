use yoyopod_protocol::ui::{
    RuntimeSnapshot, RuntimeSnapshotDomain, RuntimeSnapshotPatch, UiScreen,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotChange {
    pub domain: RuntimeSnapshotDomain,
    pub previous_app_state: UiScreen,
    pub app_state: UiScreen,
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
        current.app_state = UiScreen::Listen;
        let mut next = RuntimeSnapshot::default();
        next.app_state = UiScreen::Talk;

        let change = replace_full(&mut current, next);

        assert_eq!(change.domain, RuntimeSnapshotDomain::Full);
        assert_eq!(change.previous_app_state, UiScreen::Listen);
        assert_eq!(change.app_state, UiScreen::Talk);
        assert_eq!(current.app_state, UiScreen::Talk);
    }

    #[test]
    fn domain_patch_updates_only_that_snapshot_domain() {
        let mut current = RuntimeSnapshot::default();
        current.app_state = UiScreen::Listen;
        current.music.title = "Before".to_string();

        let change = apply_patch(
            &mut current,
            RuntimeSnapshotPatch::Music(MusicRuntimeSnapshot {
                title: "After".to_string(),
                ..MusicRuntimeSnapshot::default()
            }),
        );

        assert_eq!(change.domain, RuntimeSnapshotDomain::Music);
        assert_eq!(change.previous_app_state, UiScreen::Listen);
        assert_eq!(change.app_state, UiScreen::Listen);
        assert_eq!(current.music.title, "After");
    }

    #[test]
    fn app_state_patch_reports_old_and_new_app_state() {
        let mut current = RuntimeSnapshot::default();
        current.app_state = UiScreen::Listen;

        let change = apply_patch(
            &mut current,
            RuntimeSnapshotPatch::AppState(UiScreen::Power),
        );

        assert_eq!(change.domain, RuntimeSnapshotDomain::AppState);
        assert_eq!(change.previous_app_state, UiScreen::Listen);
        assert_eq!(change.app_state, UiScreen::Power);
    }
}
