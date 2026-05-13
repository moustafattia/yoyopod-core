use crate::components::primitives::{container, progress};
use crate::engine::Element;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressSweepProps {
    pub progress_permille: i32,
}

pub fn progress_sweep(props: ProgressSweepProps) -> Element {
    container("progress_sweep").child(progress("progress_sweep_fill", props.progress_permille))
}
