use std::ptr::NonNull;

use crate::render::lvgl::ffi;
use crate::render::styling::style as theme;

pub(crate) fn apply(obj: NonNull<ffi::lv_obj_t>, role: &'static str) -> bool {
    const SELECTOR: ffi::LvStyleSelector = 0;
    unsafe {
        match role {
            "hub_card_panel" => {
                ffi::lv_obj_set_style_pad_left(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_right(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_top(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_bottom(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 24, SELECTOR);
                ffi::lv_obj_set_style_shadow_opa(obj.as_ptr(), 76, SELECTOR);
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            "now_playing_panel"
            | "now_playing_state_chip"
            | "now_playing_progress_track"
            | "now_playing_progress_fill" => {
                ffi::lv_obj_set_style_pad_left(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_right(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_top(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_bottom(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_outline_width(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            "now_playing_icon_halo" => {
                ffi::lv_obj_set_style_border_width(obj.as_ptr(), 2, SELECTOR);
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_outline_width(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            "hub_icon_glow"
            | "footer_bar"
            | "talk_card_panel"
            | "talk_card_glow"
            | "talk_actions_primary_button"
            | "ask_icon_glow"
            | "ask_icon_halo"
            | "call_panel"
            | "call_icon_halo"
            | "call_state_chip"
            | "call_mute_badge"
            | "power_icon_halo"
            | "power_row"
            | "status_bar"
            | "status_battery_outline"
            | "listen_panel"
            | "playlist_panel"
            | "listen_empty_panel"
            | "playlist_empty_panel" => {
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            "listen_row" | "playlist_row" | "list_row" => {
                ffi::lv_obj_set_style_pad_left(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_right(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_top(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_style_pad_bottom(obj.as_ptr(), 0, SELECTOR);
                ffi::lv_obj_set_scrollbar_mode(obj.as_ptr(), ffi::LV_SCROLLBAR_MODE_OFF);
            }
            "status_gps_ring" => {
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_TRANSP, SELECTOR);
                ffi::lv_obj_set_style_radius(obj.as_ptr(), ffi::LV_RADIUS_CIRCLE, SELECTOR);
            }
            "status_gps_center" => {
                ffi::lv_obj_set_style_radius(obj.as_ptr(), ffi::LV_RADIUS_CIRCLE, SELECTOR);
            }
            "status_gps_tail" => {
                ffi::lv_obj_set_style_radius(obj.as_ptr(), 1, SELECTOR);
            }
            "status_voip_dot_left" | "status_voip_dot_after_gps" | "talk_dot" | "power_dot" => {
                ffi::lv_obj_set_style_radius(obj.as_ptr(), ffi::LV_RADIUS_CIRCLE, SELECTOR);
            }
            "hub_icon" | "ask_icon" => {
                ffi::lv_obj_set_style_image_recolor_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_image_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            _ => return false,
        }
    }
    true
}
