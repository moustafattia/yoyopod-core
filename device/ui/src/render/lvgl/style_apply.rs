use std::ptr::NonNull;

use crate::render::lvgl::ffi;
use crate::render::lvgl::style::{self as theme, WidgetStyle};

const OFFSCREEN: i32 = -4096;

pub(crate) fn apply_role_tuning_raw(obj: NonNull<ffi::lv_obj_t>, role: &'static str) {
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
            "hub_title" | "power_title" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "hub_subtitle" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_CLIP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "listen_title" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
            }
            "listen_subtitle" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
            }
            "listen_row_icon" | "playlist_row_icon" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
            }
            "listen_row_title" | "playlist_row_title" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_CLIP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_16,
                    SELECTOR,
                );
            }
            "listen_row_subtitle" | "playlist_row_subtitle" => {
                ffi::lv_label_set_long_mode(obj.as_ptr(), ffi::LV_LABEL_LONG_MODE_CLIP);
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
            }
            "listen_empty_icon" | "playlist_empty_icon" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "listen_empty_title" | "playlist_empty_title" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
            }
            "now_playing_icon_label" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
            "now_playing_state_label" => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_12,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_center(obj.as_ptr());
            }
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
            _ => {}
        }
    }
}

pub(crate) fn reset_style_raw(obj: NonNull<ffi::lv_obj_t>) {
    unsafe {
        ffi::lv_obj_remove_style_all(obj.as_ptr());
    }
}

pub(crate) fn apply_style_raw(obj: NonNull<ffi::lv_obj_t>, style: WidgetStyle) {
    const SELECTOR: ffi::LvStyleSelector = 0;

    unsafe {
        if let Some(bg_color) = style.bg_color {
            ffi::lv_obj_set_style_bg_color(
                obj.as_ptr(),
                ffi::lv_color_hex(bg_color & 0xFFFFFF),
                SELECTOR,
            );
        }
        ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), style.bg_opa, SELECTOR);

        if let Some(text_color) = style.text_color {
            ffi::lv_obj_set_style_text_color(
                obj.as_ptr(),
                ffi::lv_color_hex(text_color & 0xFFFFFF),
                SELECTOR,
            );
        }

        if let Some(border_color) = style.border_color {
            ffi::lv_obj_set_style_border_color(
                obj.as_ptr(),
                ffi::lv_color_hex(border_color & 0xFFFFFF),
                SELECTOR,
            );
        }
        ffi::lv_obj_set_style_border_width(obj.as_ptr(), style.border_width, SELECTOR);
        ffi::lv_obj_set_style_radius(obj.as_ptr(), style.radius, SELECTOR);
        ffi::lv_obj_set_style_outline_width(obj.as_ptr(), style.outline_width, SELECTOR);
        ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), style.shadow_width, SELECTOR);
    }
}

pub(crate) fn hide_widget_raw(obj: NonNull<ffi::lv_obj_t>) {
    unsafe {
        ffi::lv_obj_set_pos(obj.as_ptr(), OFFSCREEN, OFFSCREEN);
        ffi::lv_obj_set_size(obj.as_ptr(), 1, 1);
    }
}

pub(crate) fn apply_variant_raw(
    obj: NonNull<ffi::lv_obj_t>,
    role: &'static str,
    variant: &'static str,
    accent_rgb: u32,
) {
    const SELECTOR: ffi::LvStyleSelector = 0;
    unsafe {
        match (role, variant) {
            ("ask_icon_glow", "ask_listening") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 76)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), 128, SELECTOR);
                ffi::lv_obj_set_style_shadow_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 42, SELECTOR);
                ffi::lv_obj_set_style_shadow_opa(obj.as_ptr(), 102, SELECTOR);
            }
            ("ask_icon_glow", "ask_thinking") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 82)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), 51, SELECTOR);
                ffi::lv_obj_set_style_shadow_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 22, SELECTOR);
                ffi::lv_obj_set_style_shadow_opa(obj.as_ptr(), 51, SELECTOR);
            }
            ("ask_icon_glow", "ask_idle") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 82)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), 76, SELECTOR);
                ffi::lv_obj_set_style_shadow_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_shadow_width(obj.as_ptr(), 28, SELECTOR);
                ffi::lv_obj_set_style_shadow_opa(obj.as_ptr(), 76, SELECTOR);
            }
            ("ask_icon_halo", "ask_listening") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 68)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            ("ask_icon_halo", "ask_idle" | "ask_thinking") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 74)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            ("ask_title", "ask_reply") => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_18,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_LEFT, SELECTOR);
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::INK_RGB),
                    SELECTOR,
                );
            }
            ("ask_title", "ask_idle" | "ask_listening" | "ask_thinking") => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_24,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::INK_RGB),
                    SELECTOR,
                );
            }
            ("ask_subtitle", "ask_reply") => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_16,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_LEFT, SELECTOR);
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::MUTED_RGB),
                    SELECTOR,
                );
            }
            ("ask_subtitle", "ask_thinking") => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_14,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::MUTED_RGB),
                    SELECTOR,
                );
            }
            ("ask_subtitle", "ask_idle" | "ask_listening") => {
                ffi::lv_obj_set_style_text_font(
                    obj.as_ptr(),
                    &ffi::lv_font_montserrat_14,
                    SELECTOR,
                );
                ffi::lv_obj_set_style_text_align(obj.as_ptr(), ffi::LV_TEXT_ALIGN_CENTER, SELECTOR);
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
            }
            ("talk_actions_primary_button", "talk_action_primary") => {
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
            ("talk_actions_primary_button", "talk_action_selected") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
                ffi::lv_obj_set_style_border_width(obj.as_ptr(), 0, SELECTOR);
            }
            ("talk_actions_primary_button", "talk_action_unselected") => {
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
            ("talk_actions_button_label", "talk_action_primary") => {
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
            ("talk_actions_button_label", "talk_action_unselected") => {
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
            ("talk_actions_button_label", "talk_action_selected") => {
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
            ("talk_actions_status_label", "talk_action_status") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(accent_rgb),
                    SELECTOR,
                );
            }
            ("call_icon_halo", "call_halo") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 68)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            ("call_panel", "call_panel_filled") => {
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
            ("call_panel", "call_panel_outlined") => {
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
            ("call_mute_badge", "call_mute") => {
                ffi::lv_obj_set_style_bg_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::BACKGROUND_RGB, 85)),
                    SELECTOR,
                );
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
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
            ("now_playing_footer", "now_playing_playing") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(accent_rgb, theme::SURFACE_RGB, 55)),
                    SELECTOR,
                );
            }
            ("now_playing_footer", "now_playing_paused" | "now_playing_stopped") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::MUTED_RGB),
                    SELECTOR,
                );
            }
            ("now_playing_footer", "now_playing_offline") => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(theme::MUTED_DIM_RGB),
                    SELECTOR,
                );
            }
            _ => {}
        }
    }
}

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
            | "status_battery_tip" => {
                ffi::lv_obj_set_style_bg_color(obj.as_ptr(), accent, SELECTOR);
                ffi::lv_obj_set_style_bg_opa(obj.as_ptr(), theme::OPA_COVER, SELECTOR);
            }
            "status_gps_ring" | "status_battery_outline" => {
                ffi::lv_obj_set_style_border_color(obj.as_ptr(), accent, SELECTOR);
            }
            "now_playing_footer" => {
                ffi::lv_obj_set_style_text_color(
                    obj.as_ptr(),
                    ffi::lv_color_hex(mix_u24(rgb, theme::SURFACE_RGB, 55)),
                    SELECTOR,
                );
            }
            "listen_footer" | "power_footer" => {
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
fn mix_u24(primary_rgb: u32, secondary_rgb: u32, secondary_ratio_percent: u8) -> u32 {
    let secondary_ratio = u32::from(secondary_ratio_percent.min(100));
    let primary_ratio = 100 - secondary_ratio;
    let red = ((((primary_rgb >> 16) & 0xFF) * primary_ratio
        + ((secondary_rgb >> 16) & 0xFF) * secondary_ratio)
        / 100)
        & 0xFF;
    let green = ((((primary_rgb >> 8) & 0xFF) * primary_ratio
        + ((secondary_rgb >> 8) & 0xFF) * secondary_ratio)
        / 100)
        & 0xFF;
    let blue = (((primary_rgb & 0xFF) * primary_ratio + (secondary_rgb & 0xFF) * secondary_ratio)
        / 100)
        & 0xFF;
    (red << 16) | (green << 8) | blue
}

pub(crate) fn icon_label(icon_key: &str) -> String {
    if let Some(monogram) = icon_key.strip_prefix("mono:") {
        if !monogram.is_empty() {
            return monogram.to_string();
        }
    }

    let label = match icon_key {
        "playlist" | "people" | "person" | "contact" | "contacts" => "\u{f00b}",
        "ask" => "AI",
        "battery" | "setup" | "power" => "\u{f011}",
        "call_active" | "call_incoming" | "call_outgoing" | "call" | "talk" => "\u{f095}",
        "check" => "\u{f00c}",
        "clock" | "retry" | "recent" | "history" => "\u{f021}",
        "close" => "\u{f00d}",
        "listen" | "music_note" | "play" | "track" => "\u{f001}",
        "microphone" | "mic" | "voice_note" => "\u{f304}",
        "signal" | "network" => "\u{f1eb}",
        "care" | "settings" => "\u{f013}",
        "mic_off" => "X",
        _ => "\u{f00b}",
    };
    label.to_string()
}
