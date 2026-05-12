use std::ptr::NonNull;

use crate::render::lvgl::ffi;

pub(crate) fn apply(obj: NonNull<ffi::lv_obj_t>, role: &'static str) -> bool {
    const SELECTOR: ffi::LvStyleSelector = 0;
    unsafe {
        match role {
            "now_playing_title" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_WRAP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_set_style_text_line_space(obj.as_ptr(), -2, SELECTOR);
            }
            "now_playing_artist" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_DOTS);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "listen_empty_subtitle" | "playlist_empty_subtitle" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_WRAP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "ask_title" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_WRAP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "ask_subtitle" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_WRAP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_14,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "hub_footer"
            | "ask_footer"
            | "call_footer"
            | "power_footer"
            | "overlay_footer"
            | "now_playing_footer"
            | "listen_footer"
            | "playlist_footer"
            | "talk_footer"
            | "talk_actions_footer" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_CLIP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "status_network"
            | "status_signal"
            | "status_battery"
            | "status_wifi"
            | "status_time"
            | "status_battery_label" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_CLIP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "power_icon" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_align(obj.as_ptr(), ffi::LV_ALIGN_CENTER, 0, 0);
            }
            "power_row_title" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_CLIP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_LEFT, SELECTOR);
            }
            _ => return false,
        }
    }
    true
}
