use yoyopod_protocol::ui::{MusicRuntimeSnapshot, RuntimeSnapshot};

pub fn apply(current: &mut RuntimeSnapshot, snapshot: MusicRuntimeSnapshot) {
    current.music = snapshot;
}
