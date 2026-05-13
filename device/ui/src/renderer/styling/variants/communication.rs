use std::ptr::NonNull;

use crate::renderer::lvgl::ffi;
use crate::renderer::styling::style as theme;
use crate::scene::roles;

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
            (roles::TALK_ACTIONS_PRIMARY_BUTTON, "talk_action_primary") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::SURFACE_RAISED_RGB),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_width(obj.as_ptr(), 2, SELECTOR);
                ffi::lv_obj_set_style_border_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
            }
            (roles::TALK_ACTIONS_PRIMARY_BUTTON, "talk_action_selected") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_width(obj.as_ptr(), 0, SELECTOR);
            }
            (roles::TALK_ACTIONS_PRIMARY_BUTTON, "talk_action_unselected") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::SURFACE_RAISED_RGB),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_width(obj.as_ptr(), 2, SELECTOR);
                ffi::lv_obj_set_style_border_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
            }
            (roles::TALK_ACTIONS_BUTTON_LABEL, "talk_action_primary") => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_center(obj.as_ptr());
            }
            (roles::TALK_ACTIONS_BUTTON_LABEL, "talk_action_unselected") => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_center(obj.as_ptr());
            }
            (roles::TALK_ACTIONS_BUTTON_LABEL, "talk_action_selected") => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::INK_RGB),
                    SELECTOR,
                );
                ffi::lv_obj_center(obj.as_ptr());
            }
            (roles::TALK_ACTIONS_STATUS_LABEL, "talk_action_status") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
            }
            (roles::CALL_PANEL, "call_panel_filled") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_width(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_shadow_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 22, SELECTOR);
                ffi::lv_obj_set_style_shadow_opa(obj.as_ptr(), 76, SELECTOR);
            }
            (roles::CALL_PANEL, "call_panel_outlined") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 80)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_width(obj.as_ptr(), 2, SELECTOR);
                ffi::lv_obj_set_style_border_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 0, SELECTOR);
            }
            _ => return false,
        }
    }
    true
}
