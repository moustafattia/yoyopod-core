use std::ptr::NonNull;

use crate::renderer::lvgl::ffi;
use crate::renderer::styling::style as theme;
use crate::scene::roles;

pub(crate) fn apply(obj: NonNull<ffi::lv_obj_t>, role: &'static str) -> bool {
    const SELECTOR: ffi::LvStyleSelector = 0;
    unsafe {
        match role {
            roles::HUB_CARD_PANEL => {
                ffi::lv_obj_set_style_pad_left(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_right(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_top(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_bottom(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 24, SELECTOR);
                ffi::lv_obj_set_style_shadow_opa(obj.as_ptr(), 76, SELECTOR);
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            roles::NOW_PLAYING_PANEL
            | roles::NOW_PLAYING_STATE_CHIP
            | roles::NOW_PLAYING_PROGRESS_TRACK
            | roles::NOW_PLAYING_PROGRESS_FILL => {
                ffi::lv_obj_set_style_pad_left(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_right(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_top(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_bottom(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_outline_width(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            roles::NOW_PLAYING_ICON_HALO => {
                ffi::lv_obj_set_style_border_width(obj.as_ptr(), 2, SELECTOR);
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_outline_width(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            roles::HUB_ICON_GLOW
            | roles::FOOTER_BAR
            | roles::TALK_CARD_PANEL
            | roles::TALK_CARD_GLOW
            | roles::TALK_ACTIONS_PRIMARY_BUTTON
            | roles::ASK_ICON_GLOW
            | roles::ASK_ICON_HALO
            | roles::CALL_PANEL
            | roles::CALL_ICON_HALO
            | roles::CALL_STATE_CHIP
            | roles::CALL_MUTE_BADGE
            | roles::POWER_ICON_HALO
            | roles::POWER_ROW
            | roles::STATUS_BAR
            | roles::STATUS_BATTERY_OUTLINE
            | roles::LISTEN_PANEL
            | roles::PLAYLIST_PANEL
            | roles::LISTEN_EMPTY_PANEL
            | roles::PLAYLIST_EMPTY_PANEL => {
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            roles::LISTEN_ROW | roles::PLAYLIST_ROW | roles::LIST_ROW => {
                ffi::lv_obj_set_style_pad_left(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_right(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_top(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_bottom(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            roles::STATUS_GPS_RING => {
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_TRANSP, SELECTOR);
                ffi::lv_obj_set_style_radius(obj.as_ptr(), ffi::LV_RADIUS_CIRCLE, SELECTOR);
            }
            roles::STATUS_GPS_CENTER => {
                ffi::lv_obj_set_style_radius(obj.as_ptr(), ffi::LV_RADIUS_CIRCLE, SELECTOR);
            }
            roles::STATUS_GPS_TAIL => {
                ffi::lv_obj_set_style_radius(obj.as_ptr(), 1, SELECTOR);
            }
            roles::STATUS_VOIP_DOT_LEFT
            | roles::STATUS_VOIP_DOT_AFTER_GPS
            | roles::TALK_DOT
            | roles::POWER_DOT => {
                ffi::lv_obj_set_style_radius(obj.as_ptr(), ffi::LV_RADIUS_CIRCLE, SELECTOR);
            }
            roles::HUB_ICON | roles::ASK_ICON => {
                ffi::lv_obj_set_style_image_recolor_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_image_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            _ => return false,
        }
    }
    true
}
