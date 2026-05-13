use crate::components::primitives::{container, progress};
use crate::engine::Element;
use crate::roles;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressSweepProps {
    pub progress_permille: i32,
}

pub fn progress_sweep(props: ProgressSweepProps) -> Element {
    container(roles::PROGRESS_SWEEP).child(progress(
        roles::PROGRESS_SWEEP_FILL,
        props.progress_permille,
    ))
}
