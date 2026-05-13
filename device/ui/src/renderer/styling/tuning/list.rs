use std::ptr::NonNull;

use crate::renderer::lvgl::ffi;
use crate::roles;

pub(crate) fn apply(obj: NonNull<ffi::lv_obj_t>, role: &'static str) -> bool {
    const SELECTOR: ffi::LvStyleSelector = 0;
    unsafe {
        match role {
            roles::HUB_TITLE | roles::POWER_TITLE => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            roles::HUB_SUBTITLE => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_CLIP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            roles::LISTEN_TITLE => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
            }
            roles::LISTEN_SUBTITLE => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
            }
            roles::LISTEN_ROW_ICON | roles::PLAYLIST_ROW_ICON => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
            }
            roles::LISTEN_ROW_TITLE | roles::PLAYLIST_ROW_TITLE => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_CLIP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_16,
                    SELECTOR,
                );
            }
            roles::LISTEN_ROW_SUBTITLE | roles::PLAYLIST_ROW_SUBTITLE => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_CLIP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
            }
            roles::LISTEN_EMPTY_ICON | roles::PLAYLIST_EMPTY_ICON => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            roles::LISTEN_EMPTY_TITLE | roles::PLAYLIST_EMPTY_TITLE => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            roles::NOW_PLAYING_ICON_LABEL => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
            roles::NOW_PLAYING_STATE_LABEL => {
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
