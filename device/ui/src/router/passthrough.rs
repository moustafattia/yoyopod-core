use yoyopod_protocol::ui::InputAction;

use super::{PassthroughPolicy, Route, SnapshotCondition};

pub fn passthrough_policy<F>(
    route: Route,
    trigger: InputAction,
    matches_condition: F,
) -> Option<PassthroughPolicy>
where
    F: Fn(SnapshotCondition) -> bool,
{
    route
        .passthrough
        .iter()
        .find(|policy| policy.trigger == trigger && matches_condition(policy.when))
        .copied()
}

pub fn captures_button<F>(route: Route, matches_condition: F) -> bool
where
    F: Fn(SnapshotCondition) -> bool,
{
    route
        .passthrough
        .iter()
        .any(|policy| policy.captures_button && matches_condition(policy.when))
}
