mod ask;
mod communication;
mod now_playing;

use std::ptr::NonNull;

use crate::render::lvgl::ffi;

pub(crate) fn apply_variant_raw(
    obj: NonNull<ffi::lv_obj_t>,
    role: &'static str,
    variant: &'static str,
    accent_rgb: u32,
) {
    if ask::apply(obj, role, variant, accent_rgb) {
        return;
    }
    if communication::apply(obj, role, variant, accent_rgb) {
        return;
    }
    let _ = now_playing::apply(obj, role, variant, accent_rgb);
}
