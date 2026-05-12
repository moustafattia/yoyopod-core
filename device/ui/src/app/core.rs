use anyhow::Result;
use yoyopod_protocol::ui::{
    AnimationRequest, InputAction, RuntimeSnapshot, RuntimeSnapshotPatch, UiEvent, UiHealth,
    UiIntent, UiScreenChanged,
};

use crate::hardware::DisplayDevice;
use crate::presentation;
use crate::presentation::screens::ScreenModel;
use crate::presentation::transitions::TransitionSampler;
use crate::render::{Framebuffer, LvglRenderer};

use super::state::{DirtyState, UiRuntime};
use super::{input_router, navigator, snapshot, UiScreen, UiView};

pub struct RenderState {
    framebuffer: Framebuffer,
    renderer: LvglRenderer,
    frames: usize,
    last_active_screen: Option<UiScreen>,
    last_ui_renderer: String,
}

impl RenderState {
    pub fn open(width: usize, height: usize) -> Result<Self> {
        Ok(Self {
            framebuffer: Framebuffer::new(width, height),
            renderer: LvglRenderer::open(None)?,
            frames: 0,
            last_active_screen: None,
            last_ui_renderer: String::new(),
        })
    }

    pub fn frames(&self) -> usize {
        self.frames
    }

    pub fn last_ui_renderer(&self) -> &str {
        &self.last_ui_renderer
    }
}

impl UiRuntime {
    pub fn apply_snapshot(&mut self, snapshot: RuntimeSnapshot) {
        let change = snapshot::replace_full(&mut self.snapshot, snapshot);
        navigator::apply_app_state_route(self, &change.previous_app_state, &change.app_state);
        navigator::apply_runtime_preemption(self);
        navigator::clamp_focus(self);
        self.dirty.mark_full();
    }

    pub fn apply_patch(&mut self, patch: RuntimeSnapshotPatch) {
        let previous_screen = self.active_screen;
        let previous_focus = self.focus_index;
        let previous_stack_len = self.screen_stack.len();
        let change = snapshot::apply_patch(&mut self.snapshot, patch);
        navigator::apply_app_state_route(self, &change.previous_app_state, &change.app_state);
        navigator::apply_runtime_preemption(self);
        navigator::clamp_focus(self);
        self.dirty.mark_patch_domain(change.domain);
        if self.active_screen != previous_screen || self.screen_stack.len() != previous_stack_len {
            self.dirty.navigation = true;
        }
        if self.focus_index != previous_focus {
            self.dirty.focus = true;
        }
    }

    pub fn handle_input(&mut self, action: InputAction) {
        let route_state = input_router::InputRouteState {
            active_screen: self.active_screen,
            voice_note_phase: self.voice_note_phase(),
        };
        match input_router::route(action, &route_state) {
            input_router::AppCommand::AdvanceFocus => navigator::advance_focus(self),
            input_router::AppCommand::SelectFocused => navigator::select_focused(self),
            input_router::AppCommand::GoBack => navigator::go_back_or_emit(self),
            input_router::AppCommand::PttPress => navigator::handle_ptt_press(self),
            input_router::AppCommand::PttRelease => navigator::handle_ptt_release(self),
        }
        navigator::clamp_focus(self);
        self.dirty.input = true;
        self.dirty.focus = true;
    }

    pub fn start_animation(&mut self, request: AnimationRequest, started_at_ms: u64) {
        let transition = presentation::transitions::Transition::from_request(
            request,
            self.active_screen,
            self.focus_index,
            started_at_ms,
        );
        self.transitions
            .retain(|active| active.id != transition.id || active.target != transition.target);
        self.transitions.push(transition);
        self.dirty.animation = true;
    }

    pub fn advance_animations(&mut self, now_ms: u64) -> bool {
        let had_transitions = !self.transitions.is_empty();
        self.transitions
            .retain(|transition| !transition.is_complete(now_ms));
        if had_transitions {
            self.dirty.animation = true;
        }
        had_transitions
    }

    pub fn active_screen(&self) -> UiScreen {
        self.active_screen
    }

    pub fn snapshot(&self) -> &RuntimeSnapshot {
        &self.snapshot
    }

    pub fn stack(&self) -> &[UiScreen] {
        &self.screen_stack
    }

    pub fn focus_index(&self) -> usize {
        self.focus_index
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty.any()
    }

    pub fn dirty_state(&self) -> DirtyState {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = DirtyState::default();
    }

    pub fn active_transitions(&self) -> &[presentation::transitions::Transition] {
        &self.transitions
    }

    pub fn take_intents(&mut self) -> Vec<UiIntent> {
        std::mem::take(&mut self.intents)
    }

    pub fn active_view(&self) -> UiView {
        presentation::view_for_screen(
            self.active_screen,
            &self.snapshot,
            self.focus_index,
            self.selected_contact.as_ref(),
        )
    }

    pub fn active_screen_model(&self) -> ScreenModel {
        presentation::screen_model_for_screen(
            self.active_screen,
            &self.snapshot,
            self.focus_index,
            self.selected_contact.as_ref(),
        )
    }

    pub fn wants_ptt_passthrough(&self) -> bool {
        navigator::wants_ptt_passthrough(self)
    }

    pub fn health_event(&self, render: &RenderState, button_events: usize) -> UiEvent {
        UiEvent::Health(UiHealth {
            frames: render.frames(),
            button_events,
            last_ui_renderer: render.last_ui_renderer().to_string(),
            active_screen: self.active_screen_model().screen(),
        })
    }

    pub fn render_if_dirty<D>(
        &mut self,
        display: &mut D,
        render: &mut RenderState,
        now_ms: u64,
    ) -> Result<Option<UiEvent>>
    where
        D: DisplayDevice,
    {
        if !self.is_dirty() {
            return Ok(None);
        }

        let screen_model = self.active_screen_model();
        let sampler = TransitionSampler::new(&self.transitions, now_ms);
        render
            .renderer
            .render_screen_model(&mut render.framebuffer, &screen_model, &sampler)?;
        render.last_ui_renderer = "lvgl".to_string();
        display.flush_full_frame(&mut render.framebuffer)?;
        let screen_changed =
            screen_changed_if_needed(&mut render.last_active_screen, &screen_model);
        self.mark_clean();
        render.frames += 1;
        Ok(screen_changed)
    }
}

fn screen_changed_if_needed(
    last_active_screen: &mut Option<UiScreen>,
    screen_model: &ScreenModel,
) -> Option<UiEvent> {
    if last_active_screen
        .map(|screen| screen != screen_model.screen())
        .unwrap_or(true)
    {
        let event = Some(UiEvent::ScreenChanged(UiScreenChanged {
            screen: screen_model.screen(),
            title: screen_model_title(screen_model).to_string(),
        }));
        *last_active_screen = Some(screen_model.screen());
        return event;
    }
    None
}

fn screen_model_title(model: &ScreenModel) -> &str {
    match model {
        ScreenModel::Hub(hub) => hub
            .cards
            .get(hub.selected_index)
            .map(|card| card.title.as_str())
            .unwrap_or("Listen"),
        ScreenModel::Listen(list)
        | ScreenModel::Playlists(list)
        | ScreenModel::RecentTracks(list)
        | ScreenModel::Talk(list)
        | ScreenModel::Contacts(list)
        | ScreenModel::CallHistory(list) => &list.title,
        ScreenModel::NowPlaying(now_playing) => &now_playing.title,
        ScreenModel::Ask(ask) => &ask.title,
        ScreenModel::TalkContact(actions) | ScreenModel::VoiceNote(actions) => &actions.title,
        ScreenModel::IncomingCall(call)
        | ScreenModel::OutgoingCall(call)
        | ScreenModel::InCall(call) => &call.title,
        ScreenModel::Power(power) => &power.title,
        ScreenModel::Loading(overlay) | ScreenModel::Error(overlay) => &overlay.title,
    }
}
