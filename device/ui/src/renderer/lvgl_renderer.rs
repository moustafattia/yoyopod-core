use std::path::Path;

#[cfg(feature = "native-lvgl")]
use anyhow::anyhow;
use anyhow::Result;

#[path = "lvgl_list_view.rs"]
mod list_view;
#[path = "lvgl_scene.rs"]
pub mod scene;
#[path = "lvgl_scene_controller.rs"]
mod scene_controller;

use crate::animation::TransitionSampler;
use crate::engine::Mutation;
#[cfg(feature = "native-lvgl")]
use crate::engine::{ElementKind, NodeId, PropChange};
use crate::presentation::view_models::ScreenModel;
#[cfg(feature = "native-lvgl")]
use crate::renderer::lvgl::NativeLvglFacade;
#[cfg(feature = "native-lvgl")]
use crate::renderer::node_registry::NodeRegistry;
#[cfg(feature = "native-lvgl")]
use crate::renderer::widgets::{LvglFacade, WidgetId};
use crate::renderer::{
    Framebuffer, RenderMode, RenderReport, Renderer, ScreenRenderReport, ScreenRenderer,
};
#[cfg(feature = "native-lvgl")]
use scene::{NativeSceneRenderer, RustSceneBridge, SceneBridge};

#[cfg(not(feature = "native-lvgl"))]
pub struct LvglRenderer;

#[cfg(feature = "native-lvgl")]
pub struct LvglRenderer {
    renderer: RuntimeSceneLvglRenderer<RustSceneBridge<NativeLvglFacade>>,
    node_registry: NodeRegistry,
}

#[cfg(feature = "native-lvgl")]
impl LvglRenderer {
    pub fn open(explicit_source: Option<&Path>) -> Result<Self> {
        let renderer = RuntimeSceneLvglRenderer::new(RustSceneBridge::open(explicit_source)?);
        Ok(Self {
            renderer,
            node_registry: NodeRegistry::default(),
        })
    }

    pub fn render_screen_model(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        self.renderer
            .render_screen_model(framebuffer, model, transitions)
    }
}

#[cfg(feature = "native-lvgl")]
impl Renderer for LvglRenderer {
    fn apply(&mut self, mutations: &[Mutation]) -> Result<()> {
        self.renderer
            .apply_mutations(mutations, &mut self.node_registry)
    }

    fn flush(&mut self, framebuffer: &mut Framebuffer, mode: RenderMode) -> Result<RenderReport> {
        self.renderer.flush_mutations(framebuffer)?;
        Ok(RenderReport {
            renderer: "lvgl",
            mode,
            widget_count: self.node_registry.len(),
        })
    }
}

#[cfg(feature = "native-lvgl")]
impl ScreenRenderer for LvglRenderer {
    fn render(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
        dirty_region: Option<crate::engine::DirtyRegion>,
    ) -> Result<ScreenRenderReport> {
        self.render_screen_model(framebuffer, model, transitions)?;
        Ok(ScreenRenderReport {
            renderer: "lvgl",
            screen: model.screen(),
            dirty_region,
        })
    }
}

#[cfg(feature = "native-lvgl")]
trait RuntimeSceneBridge: SceneBridge {
    fn display_needs_reset(&self, framebuffer: &Framebuffer) -> bool;
    fn ensure_display_registered(&mut self, framebuffer: &Framebuffer) -> Result<()>;
    fn render_frame(&mut self, framebuffer: &mut Framebuffer) -> Result<()>;
}

#[cfg(feature = "native-lvgl")]
impl RuntimeSceneBridge for RustSceneBridge<NativeLvglFacade> {
    fn display_needs_reset(&self, framebuffer: &Framebuffer) -> bool {
        RustSceneBridge::<NativeLvglFacade>::display_needs_reset(self, framebuffer)
    }

    fn ensure_display_registered(&mut self, framebuffer: &Framebuffer) -> Result<()> {
        RustSceneBridge::<NativeLvglFacade>::ensure_display_registered(self, framebuffer)
    }

    fn render_frame(&mut self, framebuffer: &mut Framebuffer) -> Result<()> {
        RustSceneBridge::<NativeLvglFacade>::render_frame(self, framebuffer)
    }
}

#[cfg(feature = "native-lvgl")]
struct RuntimeSceneLvglRenderer<B> {
    renderer: NativeSceneRenderer<B>,
}

#[cfg(feature = "native-lvgl")]
impl<B> RuntimeSceneLvglRenderer<B>
where
    B: RuntimeSceneBridge,
{
    fn new(bridge: B) -> Self {
        Self {
            renderer: NativeSceneRenderer::new(bridge),
        }
    }

    fn render_screen_model(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
        transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        if self.renderer.bridge().display_needs_reset(framebuffer) {
            self.renderer.clear()?;
        }
        self.renderer
            .bridge_mut()
            .ensure_display_registered(framebuffer)?;
        self.renderer.render(model, transitions)?;
        self.renderer.bridge_mut().render_frame(framebuffer)
    }
}

#[cfg(feature = "native-lvgl")]
impl RuntimeSceneLvglRenderer<RustSceneBridge<NativeLvglFacade>> {
    fn apply_mutations(
        &mut self,
        mutations: &[Mutation],
        registry: &mut NodeRegistry,
    ) -> Result<()> {
        let facade = self.renderer.bridge_mut().facade_mut();
        for mutation in mutations {
            apply_mutation(facade, registry, mutation)?;
        }
        Ok(())
    }

    fn flush_mutations(&mut self, framebuffer: &mut Framebuffer) -> Result<()> {
        self.renderer
            .bridge_mut()
            .ensure_display_registered(framebuffer)?;
        self.renderer.bridge_mut().render_frame(framebuffer)
    }
}

#[cfg(feature = "native-lvgl")]
fn apply_mutation(
    facade: &mut NativeLvglFacade,
    registry: &mut NodeRegistry,
    mutation: &Mutation,
) -> Result<()> {
    match mutation {
        Mutation::Create {
            node,
            parent,
            kind,
            role,
        } => create_widget(facade, registry, *node, *parent, *kind, *role),
        Mutation::Update { node, prop } => update_widget(facade, registry, *node, prop),
        Mutation::Place { node, x, y, w, h } => {
            let widget = widget_for(registry, *node)?;
            facade.set_geometry(widget, *x, *y, *w, *h)
        }
        Mutation::Reorder { .. } => Ok(()),
        Mutation::Remove { node } => {
            if let Some(widget) = registry.remove(*node) {
                facade.destroy(widget)?;
            }
            Ok(())
        }
    }
}

#[cfg(feature = "native-lvgl")]
fn create_widget(
    facade: &mut NativeLvglFacade,
    registry: &mut NodeRegistry,
    node: NodeId,
    parent: NodeId,
    kind: ElementKind,
    role: Option<&'static str>,
) -> Result<()> {
    if registry.widget(node).is_some() {
        return Ok(());
    }
    if node == NodeId(0) || parent == node || registry.is_empty() {
        let widget = facade.create_root()?;
        registry.bind(node, widget);
        return Ok(());
    }

    let parent = widget_for(registry, parent)?;
    let role = role.ok_or_else(|| anyhow!("LVGL mutation create missing widget role"))?;
    let widget = match kind {
        ElementKind::Container | ElementKind::Progress => facade.create_container(parent, role)?,
        ElementKind::Label | ElementKind::Image => facade.create_label(parent, role)?,
    };
    registry.bind(node, widget);
    Ok(())
}

#[cfg(feature = "native-lvgl")]
fn update_widget(
    facade: &mut NativeLvglFacade,
    registry: &NodeRegistry,
    node: NodeId,
    prop: &PropChange,
) -> Result<()> {
    let widget = widget_for(registry, node)?;
    match prop {
        PropChange::Text(text) => facade.set_text(widget, text),
        PropChange::Icon(icon) => facade.set_icon(widget, icon),
        PropChange::Accent(rgb) => facade.set_accent(widget, *rgb),
        PropChange::Selected(selected) => facade.set_selected(widget, *selected),
        PropChange::Visible(visible) => facade.set_visible(widget, *visible),
        PropChange::Opacity(opacity) => facade.set_opacity(widget, *opacity),
        PropChange::Variant(variant) => facade.set_variant(widget, variant, 0),
        PropChange::Progress(value) => facade.set_progress(widget, *value),
    }
}

#[cfg(feature = "native-lvgl")]
fn widget_for(registry: &NodeRegistry, node: NodeId) -> Result<WidgetId> {
    registry
        .widget(node)
        .ok_or_else(|| anyhow!("LVGL mutation referenced unknown node {:?}", node))
}

#[cfg(not(feature = "native-lvgl"))]
impl LvglRenderer {
    pub fn open(_explicit_source: Option<&Path>) -> Result<Self> {
        anyhow::bail!("native-lvgl feature is disabled for this build")
    }

    pub fn render_screen_model(
        &mut self,
        _framebuffer: &mut Framebuffer,
        _model: &ScreenModel,
        _transitions: &TransitionSampler<'_>,
    ) -> Result<()> {
        anyhow::bail!("native-lvgl feature is disabled for this build")
    }
}

#[cfg(not(feature = "native-lvgl"))]
impl ScreenRenderer for LvglRenderer {
    fn render(
        &mut self,
        _framebuffer: &mut Framebuffer,
        _model: &ScreenModel,
        _transitions: &TransitionSampler<'_>,
        _dirty_region: Option<crate::engine::DirtyRegion>,
    ) -> Result<ScreenRenderReport> {
        anyhow::bail!("native-lvgl feature is disabled for this build")
    }
}

#[cfg(not(feature = "native-lvgl"))]
impl Renderer for LvglRenderer {
    fn apply(&mut self, _mutations: &[Mutation]) -> Result<()> {
        anyhow::bail!("native-lvgl feature is disabled for this build")
    }

    fn flush(&mut self, _framebuffer: &mut Framebuffer, _mode: RenderMode) -> Result<RenderReport> {
        anyhow::bail!("native-lvgl feature is disabled for this build")
    }
}
