use super::{Route, SelectionTarget};

pub fn selection_target(route: Route, focus_index: usize) -> Option<SelectionTarget> {
    route
        .select
        .get(focus_index)
        .or_else(|| route.select.last())
        .copied()
}
