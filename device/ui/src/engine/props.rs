use crate::render_contract::PropChange;

use super::ElementProps;

pub fn diff_props(previous: &ElementProps, next: &ElementProps, mut emit: impl FnMut(PropChange)) {
    if previous.text != next.text {
        if let Some(value) = &next.text {
            emit(PropChange::Text(value.clone()));
        }
    }
    if previous.icon_key != next.icon_key {
        if let Some(value) = &next.icon_key {
            emit(PropChange::Icon(value.clone()));
        }
    }
    if previous.accent != next.accent {
        if let Some(value) = next.accent {
            emit(PropChange::Accent(value));
        }
    }
    if previous.selected != next.selected {
        if let Some(value) = next.selected {
            emit(PropChange::Selected(value));
        }
    }
    if previous.visible != next.visible {
        if let Some(value) = next.visible {
            emit(PropChange::Visible(value));
        }
    }
    if previous.opacity != next.opacity {
        if let Some(value) = next.opacity {
            emit(PropChange::Opacity(value));
        }
    }
    if previous.offset_y != next.offset_y {
        if let Some(value) = next.offset_y {
            emit(PropChange::OffsetY(value));
        }
    }
    if previous.scale_permille != next.scale_permille {
        if let Some(value) = next.scale_permille {
            emit(PropChange::ScalePermille(value));
        }
    }
    if previous.variant != next.variant {
        if let Some(value) = next.variant {
            emit(PropChange::Variant(value));
        }
    }
    if previous.progress != next.progress {
        if let Some(value) = next.progress {
            emit(PropChange::Progress(value));
        }
    }
}
