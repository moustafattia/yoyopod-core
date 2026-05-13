use anyhow::{anyhow, bail, Result};

use super::shared::{FooterBar, StatusBarWidgets};
use super::TypedScreenController;
use crate::presentation::view_models::{CallViewModel, ScreenModel};
use crate::renderer::widgets::{roles, LvglFacade, WidgetId};
use yoyopod_protocol::ui::UiScreen;

#[derive(Default)]
pub struct CallController {
    root: Option<WidgetId>,
    status: StatusBarWidgets,
    icon_halo: Option<WidgetId>,
    panel: Option<WidgetId>,
    icon_label: Option<WidgetId>,
    title: Option<WidgetId>,
    state_chip: Option<WidgetId>,
    state_label: Option<WidgetId>,
    footer: FooterBar,
    mute_badge: Option<WidgetId>,
    mute_label: Option<WidgetId>,
}

#[derive(Clone, Copy)]
pub struct CallControllerModel<'a> {
    pub(crate) screen: UiScreen,
    pub(crate) call: &'a CallViewModel,
}

impl CallController {
    fn ensure_widgets(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
        if self.root.is_none() {
            self.root = Some(facade.create_root()?);
        }

        let root = self
            .root
            .ok_or_else(|| anyhow!("call controller missing root widget"))?;

        if self.icon_halo.is_none() {
            self.icon_halo = Some(facade.create_container(root, roles::CALL_ICON_HALO)?);
        }
        if self.panel.is_none() {
            self.panel = Some(facade.create_container(root, roles::CALL_PANEL)?);
        }
        let panel = self
            .panel
            .ok_or_else(|| anyhow!("call controller missing panel widget"))?;
        if self.icon_label.is_none() {
            self.icon_label = Some(facade.create_label(panel, roles::CALL_STATE_ICON)?);
        }
        if self.title.is_none() {
            self.title = Some(facade.create_label(root, roles::CALL_TITLE)?);
        }
        if self.state_chip.is_none() {
            self.state_chip = Some(facade.create_container(root, roles::CALL_STATE_CHIP)?);
        }
        let state_chip = self
            .state_chip
            .ok_or_else(|| anyhow!("call controller missing state chip"))?;
        if self.state_label.is_none() {
            self.state_label = Some(facade.create_label(state_chip, roles::CALL_STATE_LABEL)?);
        }
        if self.mute_badge.is_none() {
            self.mute_badge = Some(facade.create_container(root, roles::CALL_MUTE_BADGE)?);
        }
        let mute_badge = self
            .mute_badge
            .ok_or_else(|| anyhow!("call controller missing mute badge"))?;
        if self.mute_label.is_none() {
            self.mute_label = Some(facade.create_label(mute_badge, roles::CALL_MUTE_LABEL)?);
        }

        Ok(())
    }
}

impl TypedScreenController for CallController {
    type Model<'a> = CallControllerModel<'a>;

    fn model<'a>(model: &'a ScreenModel) -> Result<Self::Model<'a>> {
        call_model(model)
    }

    fn sync_model(
        &mut self,
        facade: &mut dyn LvglFacade,
        model: Self::Model<'_>,
        _transitions: &crate::animation::TransitionSampler<'_>,
    ) -> Result<()> {
        let call = model.call;
        self.ensure_widgets(facade)?;
        let accent = 0x00D4FF;

        if let Some(root) = self.root {
            self.status.sync(facade, root, &call.chrome.status, true)?;
            self.footer.sync(facade, root, &call.chrome.footer)?;
        }
        if let Some(icon_halo) = self.icon_halo {
            facade.set_variant(icon_halo, "call_halo", accent)?;
        }
        if let Some(panel) = self.panel {
            facade.set_variant(panel, call_panel_variant(model.screen), accent)?;
        }

        if let Some(title) = self.title {
            facade.set_text(title, &call.title)?;
        }
        if let Some(icon_label) = self.icon_label {
            facade.set_text(icon_label, &monogram(&call.title))?;
            facade.set_accent(icon_label, call_icon_accent(model.screen))?;
        }
        if let Some(state_chip) = self.state_chip {
            let (x, y, width) = call_state_chip_geometry(model.screen);
            facade.set_geometry(state_chip, x, y, width, 24)?;
            facade.set_accent(state_chip, call_state_accent(model.screen))?;
        }
        if let Some(state_label) = self.state_label {
            let state_text = call_state_text(model.screen, call);
            let (_, _, width) = call_state_chip_geometry(model.screen);
            facade.set_geometry(state_label, 0, 6, width, 12)?;
            facade.set_text(state_label, &state_text)?;
            facade.set_accent(state_label, call_state_accent(model.screen))?;
        }
        if let Some(mute_badge) = self.mute_badge {
            facade.set_visible(mute_badge, call.muted)?;
            facade.set_variant(mute_badge, "call_mute", 0xFF675D)?;
        }
        if let Some(mute_label) = self.mute_label {
            facade.set_text(mute_label, "MUTED")?;
        }

        Ok(())
    }

    fn teardown(&mut self, facade: &mut dyn LvglFacade) -> Result<()> {
        let root = self.root.take();
        self.status.clear();
        self.icon_halo = None;
        self.panel = None;
        self.icon_label = None;
        self.title = None;
        self.state_chip = None;
        self.state_label = None;
        self.footer.clear();
        self.mute_badge = None;
        self.mute_label = None;
        if let Some(root) = root {
            facade.destroy(root)?;
        }
        Ok(())
    }
}

fn call_model(model: &ScreenModel) -> Result<CallControllerModel<'_>> {
    match model {
        ScreenModel::IncomingCall(call)
        | ScreenModel::OutgoingCall(call)
        | ScreenModel::InCall(call) => Ok(CallControllerModel {
            screen: model.screen(),
            call,
        }),
        _ => bail!(
            "call controller received non-call screen model: {}",
            model.screen().as_str()
        ),
    }
}

fn call_state_text(screen: UiScreen, call: &CallViewModel) -> String {
    match screen {
        UiScreen::IncomingCall => "INCOMING CALL".to_string(),
        UiScreen::OutgoingCall => "CALLING...".to_string(),
        UiScreen::InCall => {
            if call.subtitle.contains("IN CALL") {
                call.subtitle.clone()
            } else if call.subtitle.trim().is_empty() {
                "IN CALL".to_string()
            } else {
                format!("IN CALL | {}", call.subtitle)
            }
        }
        _ => String::new(),
    }
}

fn call_state_accent(screen: UiScreen) -> u32 {
    match screen {
        UiScreen::InCall => 0x3DDD53,
        _ => 0x00D4FF,
    }
}

fn call_panel_variant(screen: UiScreen) -> &'static str {
    match screen {
        UiScreen::OutgoingCall => "call_panel_outlined",
        _ => "call_panel_filled",
    }
}

fn call_icon_accent(screen: UiScreen) -> u32 {
    match screen {
        UiScreen::OutgoingCall => 0x00D4FF,
        _ => 0xFFFFFF,
    }
}

fn call_state_chip_geometry(screen: UiScreen) -> (i32, i32, i32) {
    match screen {
        UiScreen::OutgoingCall => (62, 208, 116),
        UiScreen::InCall => (48, 206, 144),
        _ => (54, 208, 132),
    }
}

fn monogram(text: &str) -> String {
    let words = text.split_whitespace().collect::<Vec<_>>();
    if words.is_empty() {
        return "?".to_string();
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
        "?".to_string()
    } else {
        result
    }
}
