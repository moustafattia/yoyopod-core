use std::ptr::NonNull;

use crate::renderer::lvgl::ffi;
use crate::renderer::styling::style as theme;
use crate::scene::roles;

use super::mix_u24;

pub(crate) fn apply_accent_raw(obj: NonNull<ffi::lv_obj_t>, role: &'static str, rgb: u32) {
    const SELECTOR: ffi::LvStyleSelector = 0;
    let accent = unsafe { ffi::lv_color_hex(rgb & 0xFFFFFF) };
    unsafe {
        match role {
            roles::SCENE_BACKDROP => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::HUB_ICON_GLOW => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 72)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), 102, SELECTOR);
            }
            roles::TALK_CARD_GLOW => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 68)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), 96, SELECTOR);
            }
            roles::CALL_ICON_HALO => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 68)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::FX_HALO | roles::FX_PULSE | roles::FX_GLOW | roles::FX_SPINNER => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 70)),
                    SELECTOR,
                );
            }
            roles::HUB_CARD_PANEL | roles::TALK_CARD_PANEL | roles::CALL_PANEL => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_shadow_color(obj.as_ptr(), accent, SELECTOR);
            }
            roles::NOW_PLAYING_ICON_HALO => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 80)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 60)),
                    SELECTOR,
                );
            }
            roles::ASK_ICON_GLOW | roles::ASK_ICON_HALO | roles::TALK_ACTIONS_HEADER_BOX => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 82)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), 76, SELECTOR);
            }
            roles::CALL_STATE_CHIP => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 85)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::NOW_PLAYING_STATE_CHIP => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 65)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::TALK_ACTIONS_PRIMARY_BUTTON => {
                ffi::lv_obj_set_style_border_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::SURFACE_RAISED_RGB),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::PLAYLIST_UNDERLINE => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::NOW_PLAYING_PROGRESS_FILL => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::HUB_ICON => {
                ffi::lv_obj_set_style_image_recolor(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::INK_RGB),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_image_recolor_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::ASK_ICON => {
                ffi::lv_obj_set_style_image_recolor(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_image_recolor_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::CALL_STATE_ICON
            | roles::LIST_ROW_ICON
            | roles::LISTEN_ROW_ICON
            | roles::PLAYLIST_ROW_ICON
            | roles::NOW_PLAYING_ICON_LABEL
            | roles::POWER_ICON
            | roles::NOW_PLAYING_STATE_LABEL
            | roles::TALK_CARD_LABEL
            | roles::TALK_ACTIONS_HEADER_LABEL
            | roles::TALK_ACTIONS_BUTTON_LABEL
            | roles::TALK_ACTIONS_STATUS_LABEL
            | roles::CALL_STATE_LABEL
            | roles::STATUS_WIFI
            | roles::STATUS_TIME
            | roles::STATUS_BATTERY_LABEL => {
                ffi::lv_obj_set_style_text_color(obj.as_ptr(), accent, SELECTOR);
            }
            roles::STATUS_SIGNAL_BAR_0
            | roles::STATUS_SIGNAL_BAR_1
            | roles::STATUS_SIGNAL_BAR_2
            | roles::STATUS_SIGNAL_BAR_3
            | roles::TALK_DOT
            | roles::POWER_DOT
            | roles::STATUS_GPS_CENTER
            | roles::STATUS_GPS_TAIL
            | roles::STATUS_VOIP_DOT_LEFT
            | roles::STATUS_VOIP_DOT_AFTER_GPS
            | roles::STATUS_BATTERY_FILL
            | roles::STATUS_BATTERY_TIP
            | roles::FX_PARTICLE => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            roles::STATUS_GPS_RING | roles::STATUS_BATTERY_OUTLINE => {
                ffi::lv_obj_set_style_border_color(obj.as_ptr(), accent, SELECTOR);
            }
            roles::FOOTER_LABEL => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 65)),
                    SELECTOR,
                );
            }
            _ => {}
        }
    }
}
