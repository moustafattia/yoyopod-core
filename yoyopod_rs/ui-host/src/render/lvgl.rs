use std::path::Path;

#[cfg(not(feature = "native-lvgl"))]
use anyhow::bail;
use anyhow::Result;

use crate::framebuffer::Framebuffer;
use crate::lvgl::{LvglFacade, LvglRenderer as SemanticLvglRenderer};
#[cfg(feature = "native-lvgl")]
use crate::lvgl::{NativeLvglFacade, NativeSceneRenderer, ShimSceneBridge};
use crate::screens::ScreenModel;

#[allow(dead_code)]
pub(crate) trait RuntimeLvglBackend: LvglFacade {
    fn display_needs_reset(&self, framebuffer: &Framebuffer) -> bool;
    fn ensure_display_registered(&mut self, framebuffer: &Framebuffer) -> Result<()>;
    fn render_frame(&mut self, framebuffer: &mut Framebuffer) -> Result<()>;
}

#[cfg(feature = "native-lvgl")]
impl RuntimeLvglBackend for NativeLvglFacade {
    fn display_needs_reset(&self, framebuffer: &Framebuffer) -> bool {
        self.display_needs_reset(framebuffer)
    }

    fn ensure_display_registered(&mut self, framebuffer: &Framebuffer) -> Result<()> {
        self.ensure_display_registered(framebuffer)
    }

    fn render_frame(&mut self, framebuffer: &mut Framebuffer) -> Result<()> {
        self.render_frame(framebuffer)
    }
}

#[allow(dead_code)]
pub(crate) struct RuntimeLvglRenderer<B> {
    renderer: SemanticLvglRenderer<B>,
}

impl<B> RuntimeLvglRenderer<B>
where
    B: RuntimeLvglBackend,
{
    #[allow(dead_code)]
    pub fn render_screen_model(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
    ) -> Result<()> {
        if self.renderer.facade().display_needs_reset(framebuffer) {
            self.renderer.clear()?;
        }
        self.renderer
            .facade_mut()
            .ensure_display_registered(framebuffer)?;
        self.renderer.render(model)?;
        self.renderer.facade_mut().render_frame(framebuffer)
    }

    #[cfg(test)]
    fn from_backend_for_test(backend: B) -> Self {
        Self {
            renderer: SemanticLvglRenderer::new(backend),
        }
    }

    #[cfg(test)]
    fn backend_for_test(&self) -> &B {
        self.renderer.facade()
    }

    #[cfg(test)]
    fn backend_mut_for_test(&mut self) -> &mut B {
        self.renderer.facade_mut()
    }
}

#[cfg(not(feature = "native-lvgl"))]
pub struct LvglRenderer;

#[cfg(feature = "native-lvgl")]
pub struct LvglRenderer {
    renderer: RuntimeSceneLvglRenderer,
}

#[cfg(feature = "native-lvgl")]
impl LvglRenderer {
    pub fn open(explicit_source: Option<&Path>) -> Result<Self> {
        let bridge = ShimSceneBridge::open(explicit_source)?;
        Ok(Self {
            renderer: RuntimeSceneLvglRenderer::new(bridge),
        })
    }

    pub fn render_screen_model(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
    ) -> Result<()> {
        self.renderer.render_screen_model(framebuffer, model)
    }
}

#[cfg(feature = "native-lvgl")]
struct RuntimeSceneLvglRenderer {
    renderer: NativeSceneRenderer<ShimSceneBridge>,
}

#[cfg(feature = "native-lvgl")]
impl RuntimeSceneLvglRenderer {
    fn new(bridge: ShimSceneBridge) -> Self {
        Self {
            renderer: NativeSceneRenderer::new(bridge),
        }
    }

    fn render_screen_model(
        &mut self,
        framebuffer: &mut Framebuffer,
        model: &ScreenModel,
    ) -> Result<()> {
        if self.renderer.bridge().display_needs_reset(framebuffer) {
            self.renderer.clear()?;
        }
        self.renderer
            .bridge_mut()
            .ensure_display_registered(framebuffer)?;
        self.renderer.render(model)?;
        self.renderer.bridge_mut().render_frame(framebuffer)
    }
}

#[cfg(not(feature = "native-lvgl"))]
impl LvglRenderer {
    pub fn open(_explicit_source: Option<&Path>) -> Result<Self> {
        bail!("native-lvgl feature is disabled for this build")
    }

    pub fn render_screen_model(
        &mut self,
        _framebuffer: &mut Framebuffer,
        _model: &ScreenModel,
    ) -> Result<()> {
        bail!("native-lvgl feature is disabled for this build")
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::RuntimeLvglRenderer;
    use crate::framebuffer::Framebuffer;
    use crate::lvgl::{LvglFacade, WidgetId};
    use crate::screens::{ChromeModel, HubCardModel, HubViewModel, ScreenModel, StatusBarModel};

    #[derive(Default)]
    struct FakeBackend {
        next_id: u64,
        reset_on_next_prepare: bool,
        fail_render: bool,
        events: Vec<String>,
    }

    impl FakeBackend {
        fn mark_display_reset(&mut self) {
            self.reset_on_next_prepare = true;
        }
    }

    impl super::RuntimeLvglBackend for FakeBackend {
        fn display_needs_reset(&self, _framebuffer: &Framebuffer) -> bool {
            self.reset_on_next_prepare
        }

        fn ensure_display_registered(&mut self, _framebuffer: &Framebuffer) -> Result<()> {
            self.reset_on_next_prepare = false;
            Ok(())
        }

        fn render_frame(&mut self, _framebuffer: &mut Framebuffer) -> Result<()> {
            if self.fail_render {
                anyhow::bail!("forced render failure");
            }
            Ok(())
        }
    }

    impl LvglFacade for FakeBackend {
        fn create_root(&mut self) -> Result<WidgetId> {
            let id = WidgetId::new(self.next_id);
            self.next_id += 1;
            self.events.push(format!("create_root:{}", id.raw()));
            Ok(id)
        }

        fn create_container(&mut self, _parent: WidgetId, _role: &'static str) -> Result<WidgetId> {
            unreachable!("hub scene should not create containers")
        }

        fn create_label(&mut self, _parent: WidgetId, role: &'static str) -> Result<WidgetId> {
            let id = WidgetId::new(self.next_id);
            self.next_id += 1;
            self.events
                .push(format!("create_label:{role}:{}", id.raw()));
            Ok(id)
        }

        fn set_text(&mut self, widget: WidgetId, text: &str) -> Result<()> {
            self.events
                .push(format!("set_text:{}:{text}", widget.raw()));
            Ok(())
        }

        fn set_selected(&mut self, _widget: WidgetId, _selected: bool) -> Result<()> {
            Ok(())
        }

        fn set_icon(&mut self, _widget: WidgetId, _icon_key: &str) -> Result<()> {
            Ok(())
        }

        fn set_progress(&mut self, _widget: WidgetId, _value: i32) -> Result<()> {
            Ok(())
        }

        fn set_visible(&mut self, _widget: WidgetId, _visible: bool) -> Result<()> {
            Ok(())
        }

        fn destroy(&mut self, widget: WidgetId) -> Result<()> {
            self.events.push(format!("destroy:{}", widget.raw()));
            Ok(())
        }
    }

    #[test]
    fn runtime_wrapper_rebuilds_semantic_state_after_display_reset() -> Result<()> {
        let backend = FakeBackend::default();
        let mut renderer = RuntimeLvglRenderer::from_backend_for_test(backend);
        let mut framebuffer = Framebuffer::new(240, 280);

        renderer.render_screen_model(&mut framebuffer, &hub_screen_model("Listen"))?;
        renderer.backend_mut_for_test().mark_display_reset();
        renderer.render_screen_model(&mut framebuffer, &hub_screen_model("Talk"))?;

        assert_eq!(
            renderer.backend_for_test().events,
            vec![
                "create_root:0",
                "create_label:hub_title:1",
                "set_text:1:Listen",
                "destroy:0",
                "create_root:2",
                "create_label:hub_title:3",
                "set_text:3:Talk",
            ]
        );

        Ok(())
    }

    #[test]
    fn runtime_wrapper_propagates_backend_render_errors() -> Result<()> {
        let mut renderer = RuntimeLvglRenderer::from_backend_for_test(FakeBackend {
            fail_render: true,
            ..FakeBackend::default()
        });
        let mut framebuffer = Framebuffer::new(240, 280);

        let error = renderer
            .render_screen_model(&mut framebuffer, &hub_screen_model("Listen"))
            .expect_err("backend render failure should propagate");

        assert!(error.to_string().contains("forced render failure"));
        Ok(())
    }

    fn hub_screen_model(title: &str) -> ScreenModel {
        ScreenModel::Hub(HubViewModel {
            chrome: ChromeModel {
                status: StatusBarModel {
                    network_connected: true,
                    network_enabled: true,
                    signal_strength: 4,
                    battery_percent: 80,
                    charging: false,
                    voip_state: 1,
                },
                footer: "Footer".to_string(),
            },
            cards: vec![HubCardModel {
                key: "listen".to_string(),
                title: title.to_string(),
                subtitle: "Subtitle".to_string(),
                accent: 0x00FF88,
            }],
            selected_index: 0,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RendererMode {
    Auto,
    Lvgl,
    Framebuffer,
}

impl RendererMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Lvgl => "lvgl",
            Self::Framebuffer => "framebuffer",
        }
    }
}
