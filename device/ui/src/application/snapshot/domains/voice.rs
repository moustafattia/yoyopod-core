use yoyopod_protocol::ui::{RuntimeSnapshot, VoiceRuntimeSnapshot};

pub fn apply(current: &mut RuntimeSnapshot, snapshot: VoiceRuntimeSnapshot) {
    current.voice = snapshot;
}
