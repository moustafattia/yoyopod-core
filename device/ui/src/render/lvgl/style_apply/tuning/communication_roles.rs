use std::ptr::NonNull;

use crate::render::lvgl::ffi;

pub(crate) fn apply(obj: NonNull<ffi::lv_obj_t>, role: &'static str) -> bool {
    const SELECTOR: ffi::LvStyleSelector = 0;
    unsafe {
        match role {
            "talk_actions_header_label" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
            "talk_actions_header_name" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_DOTS);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "talk_actions_title_label" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_DOTS);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "talk_actions_button_label" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
            "talk_actions_status_label" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "call_state_icon" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
            "call_title" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_DOTS);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "call_state_label" | "call_mute_label" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
            _ => return false,
        }
    }
    true
}
