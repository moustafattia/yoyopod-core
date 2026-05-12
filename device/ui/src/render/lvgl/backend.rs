mod facade_impl;

use std::ptr::{self, NonNull};
use std::time::Instant;

use anyhow::{anyhow, Result};

use crate::render::assets::RenderAssets;
use crate::render::lvgl::ffi;
use crate::render::lvgl::flush::FlushTarget;
use crate::render::lvgl::icons;
use crate::render::lvgl::layout::LayoutResolver;
use crate::render::lvgl::roles;
use crate::render::lvgl::style::WidgetStyle;
use crate::render::lvgl::style_apply;
use crate::render::lvgl::theme::ThemeResolver;
use crate::render::lvgl::widget_factory;
use crate::render::lvgl::widget_registry::{Layout, WidgetKind, WidgetNode, WidgetRegistry};
use crate::render::lvgl::WidgetId;

const DEFAULT_WIDTH: i32 = 240;
const DEFAULT_HEIGHT: i32 = 280;

pub struct NativeLvglFacade {
    pub(crate) display: Option<NonNull<ffi::lv_display_t>>,
    pub(crate) blank_screen: Option<NonNull<ffi::lv_obj_t>>,
    pub(crate) draw_buffer: Vec<u8>,
    pub(crate) flush_target: FlushTarget,
    pub(crate) display_size: Option<(usize, usize)>,
    pub(crate) last_tick: Instant,
    pub(crate) widgets: WidgetRegistry,
    pub(crate) active_root: Option<WidgetId>,
    pub(crate) role_occurrences: std::collections::HashMap<&'static str, usize>,
    pub(crate) render_assets: RenderAssets,
}

impl NativeLvglFacade {
    pub(super) fn widget_obj(&self, widget: WidgetId) -> Result<NonNull<ffi::lv_obj_t>> {
        self.widgets.obj(widget)
    }

    pub(super) fn widget_node_mut(&mut self, widget: WidgetId) -> Result<&mut WidgetNode> {
        self.widgets.node_mut(widget)
    }

    pub(super) fn register_widget(
        &mut self,
        obj: NonNull<ffi::lv_obj_t>,
        kind: WidgetKind,
        role: &'static str,
        parent: Option<WidgetId>,
        layout: Layout,
    ) -> WidgetId {
        self.widgets.register(obj, kind, role, parent, layout)
    }

    pub(super) fn ensure_blank_screen(&mut self) -> Result<NonNull<ffi::lv_obj_t>> {
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

    pub(crate) fn invalidate_widget_registry(&mut self) {
        self.blank_screen = None;
        self.widgets.clear();
        self.active_root = None;
        self.role_occurrences.clear();
        self.flush_target.framebuffer = ptr::null_mut();
    }

    pub(super) fn apply_layout_raw(obj: NonNull<ffi::lv_obj_t>, layout: Layout) {
        unsafe {
            ffi::lv_obj_set_pos(obj.as_ptr(), layout.x, layout.y);
            ffi::lv_obj_set_size(obj.as_ptr(), layout.width.max(1), layout.height.max(1));
        }
    }

    pub(super) fn apply_node_layout_raw(
        obj: NonNull<ffi::lv_obj_t>,
        layout: Layout,
        y_offset: i32,
    ) {
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

    pub(super) fn next_role_layout(
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

    pub(super) fn style_for_role(&self, role: &'static str) -> Result<WidgetStyle> {
        ThemeResolver::new(&self.render_assets).style_for_role(role)
    }

    pub(super) fn style_for_selected_role(
        &self,
        role: &'static str,
        selected: bool,
    ) -> Result<WidgetStyle> {
        ThemeResolver::new(&self.render_assets).style_for_selected_role(role, selected)
    }
}
