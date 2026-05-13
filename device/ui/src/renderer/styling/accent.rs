use std::ptr::NonNull;

use crate::renderer::lvgl::ffi;
use crate::renderer::styling::style as theme;

use super::mix_u24;

pub(crate) fn apply_accent_raw(obj: NonNull<ffi::lv_obj_t>, role: &'static str, rgb: u32) {
    const SELECTOR: ffi::LvStyleSelector = 0;
    let accent = unsafe { ffi::lv_color_hex(rgb & 0xFFFFFF) };
    unsafe {
        match role {
            "hub_icon_glow" => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 72)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), 102, SELECTOR);
            }
            "talk_card_glow" => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 68)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), 96, SELECTOR);
            }
            "call_icon_halo" => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 68)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "fx_halo" | "fx_pulse" | "fx_glow" | "fx_spinner" => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 70)),
                    SELECTOR,
                );
            }
            "hub_card_panel" | "talk_card_panel" | "call_panel" => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_shadow_color(obj.as_ptr(), accent, SELECTOR);
            }
            "now_playing_icon_halo" => {
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
            "ask_icon_glow" | "ask_icon_halo" | "talk_actions_header_box" => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 82)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), 76, SELECTOR);
            }
            "call_state_chip" => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 85)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "now_playing_state_chip" => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::BACKGROUND_RGB, 65)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "talk_actions_primary_button" => {
                ffi::lv_obj_set_style_border_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::SURFACE_RAISED_RGB),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "playlist_underline" => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "now_playing_progress_fill" => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "hub_icon" => {
                ffi::lv_obj_set_style_image_recolor(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::INK_RGB),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_image_recolor_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "ask_icon" => {
                ffi::lv_obj_set_style_image_recolor(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_image_recolor_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "call_state_icon"
            | "list_row_icon"
            | "listen_row_icon"
            | "playlist_row_icon"
            | "power_row_icon"
            | "now_playing_icon_label"
            | "power_icon"
            | "now_playing_state_label"
            | "talk_card_label"
            | "talk_actions_header_label"
            | "talk_actions_button_label"
            | "talk_actions_status_label"
            | "call_state_label"
            | "status_wifi"
            | "status_time"
            | "status_battery_label" => {
                ffi::lv_obj_set_style_text_color(obj.as_ptr(), accent, SELECTOR);
            }
            "status_signal_bar_0"
            | "status_signal_bar_1"
            | "status_signal_bar_2"
            | "status_signal_bar_3"
            | "talk_dot"
            | "power_dot"
            | "status_gps_center"
            | "status_gps_tail"
            | "status_voip_dot_left"
            | "status_voip_dot_after_gps"
            | "status_battery_fill"
            | "status_battery_tip"
            | "fx_particle" => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "status_gps_ring" | "status_battery_outline" => {
                ffi::lv_obj_set_style_border_color(obj.as_ptr(), accent, SELECTOR);
            }
            "footer_label" => {
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
