use std::ffi::CString;

use anyhow::{Context, Result};

use super::*;
use crate::render::lvgl::LvglFacade;

impl LvglFacade for NativeLvglFacade {
    fn create_root(&mut self) -> Result<WidgetId> {
        let obj = widget_factory::create_root_object()?;
        let layout = self.layout_for_root();
        style_apply::reset_style_raw(obj);
        style_apply::apply_style_raw(obj, self.style_for_role(roles::ROOT)?);
        Self::apply_layout_raw(obj, layout);
        unsafe {
            ffi::lv_screen_load(obj.as_ptr());
        }
        self.role_occurrences.clear();
        let id = self.register_widget(obj, WidgetKind::Root, roles::ROOT, None, layout);
        self.active_root = Some(id);
        Ok(id)
    }

    fn create_container(&mut self, parent: WidgetId, role: &'static str) -> Result<WidgetId> {
        let parent_obj = self.widget_obj(parent)?;
        let obj = widget_factory::create_container_object(parent_obj, role)?;
        let layout = self.next_role_layout(Some(parent), role)?;
        style_apply::reset_style_raw(obj);
        style_apply::apply_style_raw(obj, self.style_for_role(role)?);
        Self::apply_layout_raw(obj, layout);
        style_apply::apply_role_tuning_raw(obj, role);
        Ok(self.register_widget(obj, WidgetKind::Container, role, Some(parent), layout))
    }

    fn create_label(&mut self, parent: WidgetId, role: &'static str) -> Result<WidgetId> {
        let parent_obj = self.widget_obj(parent)?;
        let (obj, kind) = widget_factory::create_label_object(parent_obj, role)?;
        let layout = self.next_role_layout(Some(parent), role)?;
        style_apply::reset_style_raw(obj);
        style_apply::apply_style_raw(obj, self.style_for_role(role)?);
        Self::apply_layout_raw(obj, layout);
        style_apply::apply_role_tuning_raw(obj, role);
        Ok(self.register_widget(obj, kind, role, Some(parent), layout))
    }

    fn set_text(&mut self, widget: WidgetId, text: &str) -> Result<()> {
        let node = self.widget_node_mut(widget)?;
        let text = CString::new(text).with_context(|| {
            format!(
                "LVGL text for widget {} contains an interior NUL byte",
                widget.raw()
            )
        })?;
        unsafe {
            ffi::lv_label_set_text(node.obj.as_ptr(), text.as_ptr());
            if matches!(
                node.role,
                "now_playing_icon_label"
                    | "now_playing_state_label"
                    | "talk_actions_header_label"
                    | "talk_actions_button_label"
                    | "call_state_icon"
                    | "call_state_label"
                    | "call_mute_label"
            ) {
                ffi::lv_obj_center(node.obj.as_ptr());
            }
        }
        Ok(())
    }

    fn set_selected(&mut self, _widget: WidgetId, _selected: bool) -> Result<()> {
        let (obj, role, layout, y_offset) = {
            let node = self.widget_node_mut(_widget)?;
            (node.obj, node.role, node.layout, node.y_offset)
        };
        style_apply::apply_style_raw(obj, self.style_for_selected_role(role, _selected)?);
        Self::apply_node_layout_raw(obj, layout, y_offset);
        Ok(())
    }

    fn set_icon(&mut self, widget: WidgetId, icon_key: &str) -> Result<()> {
        let node = self.widget_node_mut(widget)?;
        if node.kind == WidgetKind::Image {
            let descriptor = icons::descriptor_for_key(icon_key);
            unsafe {
                ffi::lv_image_set_src(
                    node.obj.as_ptr(),
                    descriptor as *const _ as *const std::ffi::c_void,
                );
                ffi::lv_obj_center(node.obj.as_ptr());
            }
            return Ok(());
        }
        self.set_text(widget, &style_apply::icon_label(icon_key))
    }

    fn set_progress(&mut self, widget: WidgetId, value: i32) -> Result<()> {
        let value = value.clamp(0, 1000);
        let node = self.widget_node_mut(widget)?;
        if node.role == "now_playing_progress_fill" {
            let fill_width = (168 * value) / 1000;
            if fill_width <= 0 {
                style_apply::hide_widget_raw(node.obj);
            } else {
                Self::apply_layout_raw(
                    node.obj,
                    Layout {
                        width: fill_width,
                        ..node.layout
                    },
                );
            }
            return Ok(());
        }
        if node.role == "status_battery_fill" {
            let fill_width = (12 * value) / 100;
            if fill_width <= 0 {
                style_apply::hide_widget_raw(node.obj);
            } else {
                Self::apply_layout_raw(
                    node.obj,
                    Layout {
                        width: fill_width,
                        ..node.layout
                    },
                );
            }
            return Ok(());
        }

        let filled = ((value as usize) * 10) / 1000;
        let empty = 10usize.saturating_sub(filled);
        let bar = format!(
            "[{}{}] {}%",
            "#".repeat(filled),
            "-".repeat(empty),
            value / 10
        );
        self.set_text(widget, &bar)
    }

    fn set_visible(&mut self, widget: WidgetId, visible: bool) -> Result<()> {
        let node = self.widget_node_mut(widget)?;
        if visible {
            Self::apply_node_layout_raw(node.obj, node.layout, node.y_offset);
        } else {
            style_apply::hide_widget_raw(node.obj);
        }
        Ok(())
    }

    fn set_opacity(&mut self, widget: WidgetId, opacity: u8) -> Result<()> {
        let node = self.widget_node_mut(widget)?;
        unsafe {
            ffi::lv_obj_set_style_opa(node.obj.as_ptr(), opacity, 0);
        }
        Ok(())
    }

    fn set_y_offset(&mut self, widget: WidgetId, offset: i32) -> Result<()> {
        let node = self.widget_node_mut(widget)?;
        node.y_offset = offset;
        Self::apply_node_layout_raw(node.obj, node.layout, node.y_offset);
        Ok(())
    }

    fn set_y(&mut self, widget: WidgetId, y: i32) -> Result<()> {
        let node = self.widget_node_mut(widget)?;
        node.layout.y = y;
        Self::apply_node_layout_raw(node.obj, node.layout, node.y_offset);
        Ok(())
    }

    fn set_geometry(
        &mut self,
        widget: WidgetId,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<()> {
        let node = self.widget_node_mut(widget)?;
        node.layout = Layout {
            x,
            y,
            width,
            height,
        };
        Self::apply_node_layout_raw(node.obj, node.layout, node.y_offset);
        Ok(())
    }

    fn set_variant(
        &mut self,
        widget: WidgetId,
        variant: &'static str,
        accent_rgb: u32,
    ) -> Result<()> {
        let node = self.widget_node_mut(widget)?;
        style_apply::apply_variant_raw(node.obj, node.role, variant, accent_rgb);
        Ok(())
    }

    fn set_accent(&mut self, widget: WidgetId, rgb: u32) -> Result<()> {
        let node = self.widget_node_mut(widget)?;
        style_apply::apply_accent_raw(node.obj, node.role, rgb);
        Ok(())
    }

    fn destroy(&mut self, widget: WidgetId) -> Result<()> {
        let obj = self.widget_obj(widget)?;
        if self.active_root == Some(widget) {
            let blank = self.ensure_blank_screen()?;
            unsafe {
                ffi::lv_screen_load(blank.as_ptr());
            }
        }
        unsafe {
            ffi::lv_obj_delete(obj.as_ptr());
        }
        self.widgets.remove_subtree(widget);
        if self.active_root == Some(widget) {
            self.active_root = None;
            self.role_occurrences.clear();
        }
        Ok(())
    }
}
