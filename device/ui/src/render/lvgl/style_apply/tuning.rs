mod base_roles;
mod communication_roles;
mod list_roles;
mod text_roles;

use std::ptr::NonNull;

use crate::render::lvgl::ffi;

pub(crate) fn apply_role_tuning_raw(obj: NonNull<ffi::lv_obj_t>, role: &'static str) {
    if base_roles::apply(obj, role) {
        return;
    }
    if list_roles::apply(obj, role) {
        return;
    }
    if communication_roles::apply(obj, role) {
        return;
    }
    let _ = text_roles::apply(obj, role);
}
