use yoyopod_protocol::ui::{RuntimeSnapshot, RuntimeSnapshotPatch};

use super::{domains, full, SnapshotChange};

pub fn apply_patch(current: &mut RuntimeSnapshot, patch: RuntimeSnapshotPatch) -> SnapshotChange {
    let previous_app_state = current.app_state;
    let domain = patch.domain();

    match patch {
        RuntimeSnapshotPatch::Full(snapshot) => {
            return full::replace_full(current, snapshot);
        }
        RuntimeSnapshotPatch::AppState(app_state) => current.app_state = app_state,
        RuntimeSnapshotPatch::Hub(hub) => domains::hub::apply(current, hub),
        RuntimeSnapshotPatch::Music(music) => domains::music::apply(current, music),
        RuntimeSnapshotPatch::Call(call) => domains::call::apply(current, call),
        RuntimeSnapshotPatch::Voice(voice) => domains::voice::apply(current, voice),
        RuntimeSnapshotPatch::Power(power) => domains::power::apply(current, power),
        RuntimeSnapshotPatch::Network(network) => domains::network::apply(current, network),
        RuntimeSnapshotPatch::Overlay(overlay) => domains::overlay::apply(current, overlay),
    }

    SnapshotChange {
        domain,
        previous_app_state,
        app_state: current.app_state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yoyopod_protocol::ui::{
        MusicRuntimeSnapshot, RuntimeSnapshotDomain, RuntimeSnapshotPatch, UiScreen,
    };

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
