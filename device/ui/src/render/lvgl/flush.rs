use std::ptr::NonNull;

use crate::render::lvgl::ffi;
use crate::render::Framebuffer;

#[derive(Default)]
pub(crate) struct FlushTarget {
    pub framebuffer: *mut Framebuffer,
}

pub(crate) unsafe extern "C" fn lvgl_flush_callback(
    display: *mut ffi::lv_display_t,
    area: *const ffi::lv_area_t,
    px_map: *mut u8,
) {
    let Some(area) = area.as_ref() else {
        flush_ready(display);
        return;
    };

    let width = (area.x2 - area.x1 + 1).max(0) as usize;
    let height = (area.y2 - area.y1 + 1).max(0) as usize;

    if width == 0 || height == 0 || px_map.is_null() {
        flush_ready(display);
        return;
    }

    let Some(display) = NonNull::new(display) else {
        return;
    };
    let target = unsafe { ffi::lv_display_get_user_data(display.as_ptr()) as *mut FlushTarget };
    if target.is_null() {
        flush_ready(display.as_ptr());
        return;
    }

    let draw_len = width * height * 2;
    let pixels = unsafe { std::slice::from_raw_parts(px_map, draw_len) };
    let target = unsafe { &mut *target };
    if !target.framebuffer.is_null() {
        let mut swapped = Vec::with_capacity(draw_len);
        for pair in pixels.chunks_exact(2) {
            swapped.push(pair[1]);
            swapped.push(pair[0]);
        }
        let framebuffer = unsafe { &mut *target.framebuffer };
        framebuffer.paste_be_bytes_region(
            area.x1.max(0) as usize,
            area.y1.max(0) as usize,
            width,
            height,
            &swapped,
        );
    }

    flush_ready(display.as_ptr());
}

fn flush_ready(display: *mut ffi::lv_display_t) {
    if let Some(display) = NonNull::new(display) {
        unsafe {
            ffi::lv_display_flush_ready(display.as_ptr());
        }
    }
}
