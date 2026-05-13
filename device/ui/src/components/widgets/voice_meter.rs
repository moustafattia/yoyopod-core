use crate::components::primitives::{container, progress};
use crate::engine::Element;
use crate::scene::roles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceMeterProps {
    pub level_permille: i32,
    pub recording: bool,
}

pub fn voice_meter(props: VoiceMeterProps) -> Element {
    container(roles::VOICE_METER)
        .selected(props.recording)
        .child(progress(roles::VOICE_METER_LEVEL, props.level_permille))
}
