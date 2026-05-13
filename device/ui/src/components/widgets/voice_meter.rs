use crate::components::primitives::{container, progress};
use crate::engine::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VoiceMeterProps {
    pub level_permille: i32,
    pub recording: bool,
}

pub fn voice_meter(props: VoiceMeterProps) -> Element {
    container("voice_meter")
        .selected(props.recording)
        .child(progress("voice_meter_level", props.level_permille))
}
