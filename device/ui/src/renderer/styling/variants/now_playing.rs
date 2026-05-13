use std::ptr::NonNull;

use crate::renderer::lvgl::ffi;
use crate::renderer::styling::style as theme;

use crate::renderer::styling::mix_u24;

pub(crate) fn apply(
    obj: NonNull<ffi::lv_obj_t>,
    role: &'static str,
    variant: &'static str,
    accent_rgb: u32,
) -> bool {
    const SELECTOR: ffi::LvStyleSelector = 0;
    unsafe {
        match (role, variant) {
            ("now_playing_icon_halo", "now_playing_playing") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 80)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 60)),
                    SELECTOR,
                );
            }
            ("now_playing_icon_halo", "now_playing_paused" | "now_playing_stopped") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(
                        theme::SURFACE_RAISED_RGB,
                        theme::BACKGROUND_RGB,
                        20,
                    )),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(theme::MUTED_RGB, theme::BACKGROUND_RGB, 60)),
                    SELECTOR,
                );
            }
            ("now_playing_icon_halo", "now_playing_offline") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(theme::ERROR_RGB, theme::BACKGROUND_RGB, 82)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(theme::ERROR_RGB, theme::BACKGROUND_RGB, 60)),
                    SELECTOR,
                );
            }
            ("now_playing_icon_label" | "now_playing_state_label", "now_playing_playing") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
            }
            (
                "now_playing_icon_label" | "now_playing_state_label",
                "now_playing_paused" | "now_playing_stopped",
            ) => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::MUTED_RGB),
                    SELECTOR,
                );
            }
            ("now_playing_icon_label", "now_playing_offline") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::INK_RGB),
                    SELECTOR,
                );
            }
            ("now_playing_state_label", "now_playing_offline") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::ERROR_RGB),
                    SELECTOR,
                );
            }
            ("now_playing_state_chip", "now_playing_playing") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 65)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            ("now_playing_state_chip", "now_playing_paused" | "now_playing_stopped") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::SURFACE_RAISED_RGB),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            ("now_playing_state_chip", "now_playing_offline") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(theme::ERROR_RGB, theme::BACKGROUND_RGB, 78)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            ("now_playing_progress_fill", "now_playing_playing") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            (
                "now_playing_progress_fill",
                "now_playing_paused" | "now_playing_stopped" | "now_playing_offline",
            ) => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::MUTED_DIM_RGB),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            ("footer_label", "now_playing_playing") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::SURFACE_RGB, 55)),
                    SELECTOR,
                );
            }
            ("footer_label", "now_playing_paused" | "now_playing_stopped") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::MUTED_RGB),
                    SELECTOR,
                );
            }
            ("footer_label", "now_playing_offline") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::MUTED_DIM_RGB),
                    SELECTOR,
                );
            }
            _ => return false,
        }
    }
    true
}
