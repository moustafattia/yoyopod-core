mod base;
mod communication;
mod text;

use std::ptr::NonNull;

use crate::renderer::lvgl::ffi;

pub(crate) fn apply_role_tuning_raw(obj: NonNull<ffi::lv_obj_t>, role: &'static str) {
    if base::apply(obj, role) {
        return;
    }
    if communication::apply(obj, role) {
        return;
    }
    let _ = text::apply(obj, role);
}
