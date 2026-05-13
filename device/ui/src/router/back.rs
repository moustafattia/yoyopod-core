use super::{BackPolicy, Route, SnapshotCondition};

pub fn back_policy<F>(route: Route, matches_condition: F) -> Option<BackPolicy>
where
    F: Fn(SnapshotCondition) -> bool,
{
    route
        .back
        .iter()
        .find(|policy| matches_condition(policy.when))
        .copied()
}
