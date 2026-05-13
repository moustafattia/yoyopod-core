use std::ptr::NonNull;

use crate::renderer::lvgl::ffi;
use crate::scene::roles;

pub(crate) fn apply(obj: NonNull<ffi::lv_obj_t>, role: &'static str) -> bool {
    const SELECTOR: ffi::LvStyleSelector = 0;
    unsafe {
        match role {
            roles::TALK_ACTIONS_HEADER_LABEL => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
            roles::TALK_ACTIONS_HEADER_NAME => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_DOTS);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            roles::TALK_ACTIONS_TITLE_LABEL => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_DOTS);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            roles::TALK_ACTIONS_BUTTON_LABEL => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
            roles::TALK_ACTIONS_STATUS_LABEL => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            roles::CALL_STATE_ICON => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
            roles::CALL_TITLE => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_DOTS);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            roles::CALL_STATE_LABEL | roles::CALL_MUTE_LABEL => {
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
