use std::ffi::CString;
use std::path::Path;
use std::ptr::{self, NonNull};
use std::time::Instant;

use anyhow::{anyhow, bail, Context, Result};

use crate::render::assets::{self, RenderAssets};
use crate::render::lvgl::ffi;
use crate::render::lvgl::icons;
use crate::render::lvgl::layout::LayoutResolver;
use crate::render::lvgl::roles;
use crate::render::lvgl::style::WidgetStyle;
use crate::render::lvgl::style_apply;
use crate::render::lvgl::theme::ThemeResolver;
use crate::render::lvgl::widget_factory;
use crate::render::lvgl::widget_registry::{Layout, WidgetKind, WidgetNode, WidgetRegistry};
use crate::render::lvgl::{LvglFacade, WidgetId};
use crate::render::Framebuffer;

const DEFAULT_WIDTH: i32 = 240;
const DEFAULT_HEIGHT: i32 = 280;
const DRAW_BUFFER_ROWS: usize = 40;

#[derive(Default)]
struct FlushTarget {
    framebuffer: *mut Framebuffer,
}

pub struct NativeLvglFacade {
    display: Option<NonNull<ffi::lv_display_t>>,
    blank_screen: Option<NonNull<ffi::lv_obj_t>>,
    draw_buffer: Vec<u8>,
    flush_target: FlushTarget,
    display_size: Option<(usize, usize)>,
    last_tick: Instant,
    widgets: WidgetRegistry,
    active_root: Option<WidgetId>,
    role_occurrences: std::collections::HashMap<&'static str, usize>,
    render_assets: RenderAssets,
}

impl NativeLvglFacade {
    pub fn open(explicit_source: Option<&Path>) -> Result<Self> {
        validate_explicit_source_dir(explicit_source)?;
        let render_assets = assets::load_render_assets().context("loading LVGL render assets")?;
        unsafe {
            ffi::lv_init();
        }

        Ok(Self {
            display: None,
            blank_screen: None,
            draw_buffer: Vec::new(),
            flush_target: FlushTarget::default(),
            display_size: None,
            last_tick: Instant::now(),
            widgets: WidgetRegistry::default(),
            active_root: None,
            role_occurrences: std::collections::HashMap::new(),
            render_assets,
        })
    }

    pub(crate) fn display_needs_reset(&self, framebuffer: &Framebuffer) -> bool {
        let size = (framebuffer.width(), framebuffer.height());
        self.display.is_some() && self.display_size != Some(size)
    }

    pub(crate) fn ensure_display_registered(&mut self, framebuffer: &Framebuffer) -> Result<()> {
        let size = (framebuffer.width(), framebuffer.height());
        if self.display_size == Some(size) && self.display.is_some() {
            return Ok(());
        }

        if let Some(display) = self.display.take() {
            unsafe {
                ffi::lv_display_delete(display.as_ptr());
            }
        }
        self.invalidate_widget_registry();
        self.display_size = Some(size);

        let display = unsafe { ffi::lv_display_create(size.0 as i32, size.1 as i32) };
        let display =
            NonNull::new(display).ok_or_else(|| anyhow!("LVGL display creation failed"))?;

        self.draw_buffer = vec![0; size.0 * DRAW_BUFFER_ROWS * 2];
        unsafe {
            ffi::lv_display_set_default(display.as_ptr());
            ffi::lv_display_set_flush_cb(display.as_ptr(), Some(lvgl_flush_callback));
            ffi::lv_display_set_user_data(
                display.as_ptr(),
                &mut self.flush_target as *mut FlushTarget as *mut _,
            );
            ffi::lv_display_set_buffers(
                display.as_ptr(),
                self.draw_buffer.as_mut_ptr().cast(),
                ptr::null_mut(),
                self.draw_buffer.len() as u32,
                ffi::LV_DISPLAY_RENDER_MODE_PARTIAL,
            );
        }

        self.display = Some(display);
        Ok(())
    }

    pub(crate) fn render_frame(&mut self, framebuffer: &mut Framebuffer) -> Result<()> {
        self.ensure_display_registered(framebuffer)?;
        self.flush_target.framebuffer = framebuffer as *mut Framebuffer;

        if let Some(root) = self.active_root {
            let root_obj = self.widget_obj(root)?;
            unsafe {
                ffi::lv_obj_invalidate(root_obj.as_ptr());
            }
        } else if let Some(display) = self.display {
            let active = unsafe { ffi::lv_display_get_screen_active(display.as_ptr()) };
            if let Some(active) = NonNull::new(active) {
                unsafe {
                    ffi::lv_obj_invalidate(active.as_ptr());
                }
            }
        }

        let elapsed_ms = self
            .last_tick
            .elapsed()
            .as_millis()
            .min(u128::from(u32::MAX)) as u32;
        self.last_tick = Instant::now();
        unsafe {
            ffi::lv_tick_inc(elapsed_ms.max(1));
            let _ = ffi::lv_timer_handler();
        }
        Ok(())
    }

    fn widget_obj(&self, widget: WidgetId) -> Result<NonNull<ffi::lv_obj_t>> {
        self.widgets.obj(widget)
    }

    fn widget_node_mut(&mut self, widget: WidgetId) -> Result<&mut WidgetNode> {
        self.widgets.node_mut(widget)
    }

    fn register_widget(
        &mut self,
        obj: NonNull<ffi::lv_obj_t>,
        kind: WidgetKind,
        role: &'static str,
        parent: Option<WidgetId>,
        layout: Layout,
    ) -> WidgetId {
        self.widgets.register(obj, kind, role, parent, layout)
    }

    fn ensure_blank_screen(&mut self) -> Result<NonNull<ffi::lv_obj_t>> {
        if let Some(blank) = self.blank_screen {
            return Ok(blank);
        }

        let blank = unsafe { ffi::lv_obj_create(ptr::null_mut()) };
        let blank =
            NonNull::new(blank).ok_or_else(|| anyhow!("LVGL blank screen creation failed"))?;
        let size = self
            .display_size
            .map(|(width, height)| (width as i32, height as i32))
            .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT));
        style_apply::reset_style_raw(blank);
        style_apply::apply_style_raw(blank, self.style_for_role(roles::ROOT)?);
        Self::apply_layout_raw(
            blank,
            Layout {
                x: 0,
                y: 0,
                width: size.0,
                height: size.1,
            },
        );
        self.blank_screen = Some(blank);
        Ok(blank)
    }

    fn invalidate_widget_registry(&mut self) {
        self.blank_screen = None;
        self.widgets.clear();
        self.active_root = None;
        self.role_occurrences.clear();
        self.flush_target.framebuffer = ptr::null_mut();
    }

    fn apply_layout_raw(obj: NonNull<ffi::lv_obj_t>, layout: Layout) {
        unsafe {
            ffi::lv_obj_set_pos(obj.as_ptr(), layout.x, layout.y);
            ffi::lv_obj_set_size(obj.as_ptr(), layout.width.max(1), layout.height.max(1));
        }
    }

    fn apply_node_layout_raw(obj: NonNull<ffi::lv_obj_t>, layout: Layout, y_offset: i32) {
        Self::apply_layout_raw(
            obj,
            Layout {
                y: layout.y + y_offset,
                ..layout
            },
        );
    }

    fn layout_for_role_asset(&self, role: &'static str, occurrence: usize) -> Option<Layout> {
        LayoutResolver::new(&self.render_assets)
            .resolve_role(role, occurrence)
            .map(|layout| Layout {
                x: layout.x,
                y: layout.y,
                width: layout.width,
                height: layout.height,
            })
    }

    fn layout_for_root(&self) -> Layout {
        let (width, height) = self
            .display_size
            .map(|(width, height)| (width as i32, height as i32))
            .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT));
        Layout {
            x: 0,
            y: 0,
            width,
            height,
        }
    }

    fn next_role_layout(
        &mut self,
        _parent: Option<WidgetId>,
        role: &'static str,
    ) -> Result<Layout> {
        let occurrence = *self.role_occurrences.entry(role).or_insert(0);
        if let Some(layout) = self.layout_for_role_asset(role, occurrence) {
            self.role_occurrences.insert(role, occurrence + 1);
            return Ok(layout);
        }

        anyhow::bail!("missing LVGL layout asset for role {role}")
    }

    fn style_for_role(&self, role: &'static str) -> Result<WidgetStyle> {
        ThemeResolver::new(&self.render_assets).style_for_role(role)
    }

    fn style_for_selected_role(&self, role: &'static str, selected: bool) -> Result<WidgetStyle> {
        ThemeResolver::new(&self.render_assets).style_for_selected_role(role, selected)
    }
}

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

impl Drop for NativeLvglFacade {
    fn drop(&mut self) {
        if let Some(root) = self.active_root.take() {
            if let Ok(obj) = self.widgets.obj(root) {
                unsafe {
                    ffi::lv_obj_delete(obj.as_ptr());
                }
            }
        }
        if let Some(blank) = self.blank_screen.take() {
            unsafe {
                ffi::lv_obj_delete(blank.as_ptr());
            }
        }
        self.invalidate_widget_registry();
        if let Some(display) = self.display.take() {
            unsafe {
                ffi::lv_display_delete(display.as_ptr());
            }
        }
        unsafe {
            ffi::lv_deinit();
        }
    }
}

fn validate_explicit_source_dir(explicit_source: Option<&Path>) -> Result<()> {
    if let Some(source) = explicit_source {
        if source.exists() {
            return Ok(());
        }
        bail!("LVGL source directory not found at {}", source.display());
    }

    Ok(())
}

unsafe extern "C" fn lvgl_flush_callback(
    display: *mut ffi::lv_display_t,
    area: *const ffi::lv_area_t,
    px_map: *mut u8,
) {
    let Some(area) = area.as_ref() else {
        if let Some(display) = NonNull::new(display) {
            unsafe {
                ffi::lv_display_flush_ready(display.as_ptr());
            }
        }
        return;
    };

    let width = (area.x2 - area.x1 + 1).max(0) as usize;
    let height = (area.y2 - area.y1 + 1).max(0) as usize;

    if width == 0 || height == 0 || px_map.is_null() {
        if let Some(display) = NonNull::new(display) {
            unsafe {
                ffi::lv_display_flush_ready(display.as_ptr());
            }
        }
        return;
    }

    let Some(display) = NonNull::new(display) else {
        return;
    };
    let target = unsafe { ffi::lv_display_get_user_data(display.as_ptr()) as *mut FlushTarget };
    if target.is_null() {
        unsafe {
            ffi::lv_display_flush_ready(display.as_ptr());
        }
        return;
    }

    let draw_len = width * height * 2;
    let pixels = unsafe { std::slice::from_raw_parts(px_map, draw_len) };
    let target = unsafe { &mut *target };
    if !target.framebuffer.is_null() {
        let mut swapped = Vec::with_capacity(draw_len);
        for pair in pixels.chunks_exact(2) {
            swapped.push(pair[1]);
            swapped.push(pair[0]);
        }
        let framebuffer = unsafe { &mut *target.framebuffer };
        framebuffer.paste_be_bytes_region(
            area.x1.max(0) as usize,
            area.y1.max(0) as usize,
            width,
            height,
            &swapped,
        );
    }

    unsafe {
        ffi::lv_display_flush_ready(display.as_ptr());
    }
}
