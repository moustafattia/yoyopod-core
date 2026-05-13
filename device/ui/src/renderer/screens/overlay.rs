use anyhow::{anyhow, bail, Result};

use super::shared::{FooterBar, StatusBarWidgets};
use super::TypedScreenController;
use crate::presentation::view_models::{OverlayViewModel, ScreenModel};
use crate::renderer::widgets::{roles, LvglFacade, WidgetId};

#[derive(Default)]
pub struct OverlayController {
    root: Option<WidgetId>,
    status: StatusBarWidgets,
    title: Option<WidgetId>,
    subtitle: Option<WidgetId>,
    footer: FooterBar,
}

impl OverlayController {
    fn ensure_widgets(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
        if self.root.is_none() {
            self.root = Some(facade.create_root()?);
        }

        let root = self
            .root
            .ok_or_else(|| anyhow!("overlay controller missing root widget"))?;

        if self.title.is_none() {
            self.title = Some(facade.create_label(root, roles::OVERLAY_TITLE)?);
        }
        if self.subtitle.is_none() {
            self.subtitle = Some(facade.create_label(root, roles::OVERLAY_SUBTITLE)?);
        }

        Ok(())
    }
}

impl TypedScreenController for OverlayController {
    type Model<'a> = &'a OverlayViewModel;

    fn model<'a>(model: &'a ScreenModel) -> Result<Self::Model<'a>> {
        overlay_model(model)
    }

    fn sync_model(
        &mut self,
        facade: &mut dyn LvglFacade,
        overlay: Self::Model<'_>,
        _transitions: &crate::animation::TransitionSampler<'_>,
    ) -> Result<()> {
        self.ensure_widgets(facade)?;
        if let Some(root) = self.root {
            self.status
                .sync(facade, root, &overlay.chrome.status, true)?;
            self.footer.sync(facade, root, &overlay.chrome.footer)?;
        }

        if let Some(title) = self.title {
            facade.set_text(title, &overlay.title)?;
        }
        if let Some(subtitle) = self.subtitle {
            facade.set_text(subtitle, &overlay.subtitle)?;
        }

        Ok(())
    }

    fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
        let root = self.root.take();
        self.status.clear();
        self.title = None;
        self.subtitle = None;
        self.footer.clear();
        if let Some(root) = root {
            facade.destroy(root)?;
        }
        Ok(())
    }
}

fn overlay_model(model: &ScreenModel) -> Result<&OverlayViewModel> {
    match model {
        ScreenModel::Loading(overlay) | ScreenModel::Error(overlay) => Ok(overlay),
        _ => bail!(
            "overlay controller received non-overlay screen model: {}",
            model.screen().as_str()
        ),
    }
}
