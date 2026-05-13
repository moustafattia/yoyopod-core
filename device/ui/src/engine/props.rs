use crate::render_contract::PropChange;

use super::ElementProps;

pub fn diff_props(previous: &ElementProps, next: &ElementProps, out: &mut Vec<PropChange>) {
    if previous.text != next.text {
        if let Some(value) = &next.text {
            out.push(PropChange::Text(value.clone()));
        }
    }
    if previous.icon_key != next.icon_key {
        if let Some(value) = &next.icon_key {
            out.push(PropChange::Icon(value.clone()));
        }
    }
    if previous.accent != next.accent {
        if let Some(value) = next.accent {
            out.push(PropChange::Accent(value));
        }
    }
    if previous.selected != next.selected {
        if let Some(value) = next.selected {
            out.push(PropChange::Selected(value));
        }
    }
    if previous.visible != next.visible {
        if let Some(value) = next.visible {
            out.push(PropChange::Visible(value));
        }
    }
    if previous.opacity != next.opacity {
        if let Some(value) = next.opacity {
            out.push(PropChange::Opacity(value));
        }
    }
    if previous.variant != next.variant {
        if let Some(value) = next.variant {
            out.push(PropChange::Variant(value));
        }
    }
    if previous.progress != next.progress {
        if let Some(value) = next.progress {
            out.push(PropChange::Progress(value));
        }
    }
}
