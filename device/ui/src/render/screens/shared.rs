pub(crate) use super::status_bar::StatusBarWidgets;

use anyhow::{anyhow, Result};

use crate::render::widgets::{roles, LvglFacade, WidgetId};

#[derive(Default)]
pub(crate) struct FooterBar {
    bar: Option<WidgetId>,
    label: Option<WidgetId>,
}

impl FooterBar {
    pub(crate) fn sync(
        &mut self,
        facade: &mut dyn LvglFacade,
        root: WidgetId,
        label_role: &'static str,
        text: &str,
    ) -> Result<()> {
        self.ensure_widgets(facade, root, label_role)?;
        if let Some(label) = self.label {
            facade.set_text(label, text)?;
            facade.set_visible(label, !text.trim().is_empty())?;
        }
        Ok(())
    }

    fn ensure_widgets(
        &mut self,
        facade: &mut dyn LvglFacade,
        root: WidgetId,
        label_role: &'static str,
    ) -> Result<()> {
        if self.bar.is_none() {
            self.bar = Some(facade.create_container(root, roles::FOOTER_BAR)?);
        }
        let bar = self
            .bar
            .ok_or_else(|| anyhow!("footer bar missing root widget"))?;

        if self.label.is_none() {
            self.label = Some(facade.create_label(bar, label_role)?);
        }

        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}

#[derive(Default)]
pub(crate) struct FooterLabel {
    label: Option<WidgetId>,
}

impl FooterLabel {
    pub(crate) fn sync(
        &mut self,
        facade: &mut dyn LvglFacade,
        root: WidgetId,
        label_role: &'static str,
        text: &str,
    ) -> Result<()> {
        if self.label.is_none() {
            self.label = Some(facade.create_label(root, label_role)?);
        }
        if let Some(label) = self.label {
            facade.set_text(label, text)?;
            facade.set_visible(label, !text.trim().is_empty())?;
        }
        Ok(())
    }

    pub(crate) fn sync_with_accent(
        &mut self,
        facade: &mut dyn LvglFacade,
        root: WidgetId,
        label_role: &'static str,
        text: &str,
        accent: u32,
    ) -> Result<()> {
        self.sync(facade, root, label_role, text)?;
        if let Some(label) = self.label {
            facade.set_accent(label, accent)?;
        }
        Ok(())
    }

    pub(crate) fn sync_with_variant(
        &mut self,
        facade: &mut dyn LvglFacade,
        root: WidgetId,
        label_role: &'static str,
        text: &str,
        variant: &'static str,
        accent: u32,
    ) -> Result<()> {
        self.sync(facade, root, label_role, text)?;
        if let Some(label) = self.label {
            facade.set_variant(label, variant, accent)?;
        }
        Ok(())
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
}
