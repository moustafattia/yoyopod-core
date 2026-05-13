use anyhow::{anyhow, bail, Result};

mod layout;

use super::shared::{FooterBar, StatusBarWidgets};
use super::TypedScreenController;
use crate::presentation::view_models::{ScreenModel, TalkActionsViewModel};
use crate::renderer::widgets::{roles, LvglFacade, WidgetId};

const ACCENT: u32 = 0x00D4FF;
const BACKGROUND: u32 = 0x2A2D35;
const SUCCESS: u32 = 0x3DDD53;
const WARNING: u32 = 0xFFD549;
const ERROR: u32 = 0xFF675D;
const NEUTRAL: u32 = 0x9CA3AF;

#[derive(Default)]
pub struct TalkActionsController {
    root: Option<WidgetId>,
    status: StatusBarWidgets,
    header_box: Option<WidgetId>,
    header_label: Option<WidgetId>,
    header_name: Option<WidgetId>,
    buttons: Vec<WidgetId>,
    button_labels: Vec<WidgetId>,
    title_label: Option<WidgetId>,
    status_label: Option<WidgetId>,
    dots: Vec<WidgetId>,
    footer: FooterBar,
}

impl TalkActionsController {
    fn ensure_widgets(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
        if self.root.is_none() {
            self.root = Some(facade.create_root()?);
        }
        let root = self
            .root
            .ok_or_else(|| anyhow!("talk-actions controller missing root widget"))?;

        if self.header_box.is_none() {
            self.header_box = Some(facade.create_container(root, roles::TALK_ACTIONS_HEADER_BOX)?);
        }
        let header_box = self
            .header_box
            .ok_or_else(|| anyhow!("talk-actions controller missing header"))?;
        if self.header_label.is_none() {
            self.header_label =
                Some(facade.create_label(header_box, roles::TALK_ACTIONS_HEADER_LABEL)?);
        }
        if self.header_name.is_none() {
            self.header_name = Some(facade.create_label(root, roles::TALK_ACTIONS_HEADER_NAME)?);
        }
        while self.buttons.len() < 3 {
            let button = facade.create_container(root, roles::TALK_ACTIONS_PRIMARY_BUTTON)?;
            self.buttons.push(button);
            self.button_labels
                .push(facade.create_label(button, roles::TALK_ACTIONS_BUTTON_LABEL)?);
        }
        if self.title_label.is_none() {
            self.title_label = Some(facade.create_label(root, roles::TALK_ACTIONS_TITLE_LABEL)?);
        }
        if self.status_label.is_none() {
            self.status_label = Some(facade.create_label(root, roles::TALK_ACTIONS_STATUS_LABEL)?);
        }
        while self.dots.len() < 3 {
            self.dots
                .push(facade.create_container(root, roles::TALK_DOT)?);
        }
        Ok(())
    }
}

impl TypedScreenController for TalkActionsController {
    type Model<'a> = &'a TalkActionsViewModel;

    fn model<'a>(model: &'a ScreenModel) -> Result<Self::Model<'a>> {
        talk_actions_model(model)
    }

    fn sync_model(
        &mut self,
        facade: &mut dyn LvglFacade,
        actions: Self::Model<'_>,
        _transitions: &crate::animation::TransitionSampler<'_>,
    ) -> Result<()> {
        self.ensure_widgets(facade)?;
        if let Some(root) = self.root {
            self.status
                .sync(facade, root, &actions.chrome.status, true)?;
            self.footer.sync(facade, root, &actions.chrome.footer)?;
        }
        if let Some(header_box) = self.header_box {
            facade.set_accent(header_box, ACCENT)?;
        }
        if let Some(header_label) = self.header_label {
            facade.set_text(header_label, monogram(&actions.contact_name).as_str())?;
            facade.set_accent(header_label, ACCENT)?;
        }
        if let Some(header_name) = self.header_name {
            facade.set_text(header_name, &actions.contact_name)?;
        }

        let action_count = actions.buttons.len().min(3);
        let selected_index = actions.selected_index.min(action_count.saturating_sub(1));
        if actions.layout_kind == 1 {
            self.sync_primary_layout(facade, actions, action_count)?;
        } else {
            self.sync_action_row(facade, actions, action_count, selected_index)?;
        }
        Ok(())
    }

    fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
        let root = self.root.take();
        self.status.clear();
        self.header_box = None;
        self.header_label = None;
        self.header_name = None;
        self.buttons.clear();
        self.button_labels.clear();
        self.title_label = None;
        self.status_label = None;
        self.dots.clear();
        self.footer.clear();
        if let Some(root) = root {
            facade.destroy(root)?;
        }
        Ok(())
    }
}
fn talk_actions_model(model: &ScreenModel) -> Result<&TalkActionsViewModel> {
    match model {
        ScreenModel::TalkContact(actions) | ScreenModel::VoiceNote(actions) => Ok(actions),
        _ => bail!(
            "talk-actions controller received non-talk-action screen model: {}",
            model.screen().as_str()
        ),
    }
}
fn monogram(text: &str) -> String {
    let words = text.split_whitespace().collect::<Vec<_>>();
    if words.is_empty() {
        return "T".to_string();
    }

    let mut result = String::new();
    if words.len() > 1 {
        for word in words.iter().take(2) {
            if let Some(letter) = word.chars().next() {
                result.push(letter.to_ascii_uppercase());
            }
        }
    } else {
        for letter in words[0].chars().take(2) {
            result.push(letter.to_ascii_uppercase());
        }
    }

    if result.is_empty() {
        "T".to_string()
    } else {
        result
    }
}
