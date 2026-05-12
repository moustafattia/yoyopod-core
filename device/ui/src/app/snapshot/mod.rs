mod domains;
mod full;
mod patch;

use yoyopod_protocol::ui::{RuntimeSnapshotDomain, UiScreen};

pub use full::replace_full;
pub use patch::apply_patch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SnapshotChange {
    pub domain: RuntimeSnapshotDomain,
    pub previous_app_state: UiScreen,
    pub app_state: UiScreen,
}
