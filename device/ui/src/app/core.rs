use crate::presentation;
use crate::presentation::registry::{screen_entry, FocusPolicy, NavigationPolicy};
use crate::presentation::screens;
use crate::presentation::screens::ScreenModel;
use crate::presentation::transitions::Transition;
use yoyopod_protocol::ui::{
    AnimationRequest, CallIntent, InputAction, ListItemSnapshot, MusicIntent, RuntimeSnapshot,
    RuntimeSnapshotDomain, RuntimeSnapshotPatch, UiIntent, VoiceFileAction, VoiceIntent,
    VoiceRecipientAction,
};

use super::{focus, input_router, intents, navigator, snapshot, UiScreen, UiView};

#[derive(Debug, Clone)]
pub struct UiRuntime {
    snapshot: RuntimeSnapshot,
    active_screen: UiScreen,
    screen_stack: Vec<UiScreen>,
    focus_index: usize,
    intents: Vec<UiIntent>,
    dirty: DirtyState,
    selected_contact: Option<ListItemSnapshot>,
    transitions: Vec<Transition>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DirtyState {
    pub full: bool,
    pub app_state: bool,
    pub hub: bool,
    pub music: bool,
    pub call: bool,
    pub voice: bool,
    pub power: bool,
    pub network: bool,
    pub overlay: bool,
    pub navigation: bool,
    pub focus: bool,
    pub input: bool,
    pub animation: bool,
}

impl DirtyState {
    pub fn any(self) -> bool {
        self.full
            || self.app_state
            || self.hub
            || self.music
            || self.call
            || self.voice
            || self.power
            || self.network
            || self.overlay
            || self.navigation
            || self.focus
            || self.input
            || self.animation
    }

    fn mark_full(&mut self) {
        self.full = true;
        self.app_state = true;
        self.hub = true;
        self.music = true;
        self.call = true;
        self.voice = true;
        self.power = true;
        self.network = true;
        self.overlay = true;
        self.navigation = true;
        self.focus = true;
    }

    fn mark_patch_domain(&mut self, domain: RuntimeSnapshotDomain) {
        match domain {
            RuntimeSnapshotDomain::Full => self.mark_full(),
            RuntimeSnapshotDomain::AppState => {
                self.app_state = true;
                self.navigation = true;
            }
            RuntimeSnapshotDomain::Hub => self.hub = true,
            RuntimeSnapshotDomain::Music => self.music = true,
            RuntimeSnapshotDomain::Call => self.call = true,
            RuntimeSnapshotDomain::Voice => self.voice = true,
            RuntimeSnapshotDomain::Power => self.power = true,
            RuntimeSnapshotDomain::Network => self.network = true,
            RuntimeSnapshotDomain::Overlay => self.overlay = true,
        }
    }
}

impl Default for UiRuntime {
    fn default() -> Self {
        Self {
            snapshot: RuntimeSnapshot::default(),
            active_screen: UiScreen::Hub,
            screen_stack: Vec::new(),
            focus_index: 0,
            intents: Vec::new(),
            dirty: {
                let mut dirty = DirtyState::default();
                dirty.mark_full();
                dirty
            },
            selected_contact: None,
            transitions: Vec::new(),
        }
    }
}

impl UiRuntime {
    pub fn apply_snapshot(&mut self, snapshot: RuntimeSnapshot) {
        let change = snapshot::replace_full(&mut self.snapshot, snapshot);
        self.apply_app_state_route(&change.previous_app_state, &change.app_state);
        self.apply_runtime_preemption();
        self.clamp_focus();
        self.dirty.mark_full();
    }

    pub fn apply_patch(&mut self, patch: RuntimeSnapshotPatch) {
        let previous_screen = self.active_screen;
        let previous_focus = self.focus_index;
        let previous_stack_len = self.screen_stack.len();
        let change = snapshot::apply_patch(&mut self.snapshot, patch);
        self.apply_app_state_route(&change.previous_app_state, &change.app_state);
        self.apply_runtime_preemption();
        self.clamp_focus();
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
            input_router::AppCommand::AdvanceFocus => self.advance_focus(),
            input_router::AppCommand::SelectFocused => self.select_focused(),
            input_router::AppCommand::GoBack => self.go_back_or_emit(),
            input_router::AppCommand::PttPress => self.handle_ptt_press(),
            input_router::AppCommand::PttRelease => self.handle_ptt_release(),
        }
        self.clamp_focus();
        self.dirty.input = true;
        self.dirty.focus = true;
    }

    pub fn start_animation(&mut self, request: AnimationRequest, started_at_ms: u64) {
        let transition =
            Transition::from_request(request, self.active_screen, self.focus_index, started_at_ms);
        self.transitions
            .retain(|active| active.id != transition.id || active.target != transition.target);
        self.transitions.push(transition);
        self.dirty.animation = true;
    }

    pub fn advance_animations(&mut self, now_ms: u64) -> bool {
        let had_transitions = !self.transitions.is_empty();
        self.transitions
            .retain(|transition| !transition.is_complete(now_ms));
        let changed = had_transitions;
        if changed {
            self.dirty.animation = true;
        }
        changed
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

    pub fn active_transitions(&self) -> &[Transition] {
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

    fn apply_runtime_preemption(&mut self) {
        if let Some(screen) = navigator::runtime_preemption(&self.snapshot) {
            if self.active_screen != screen {
                self.push_screen(screen);
            }
            return;
        }

        if navigator::is_overlay_screen(self.active_screen) {
            self.pop_until_not_overlay();
        }

        if navigator::is_call_screen(self.active_screen) && self.snapshot.call.state == "idle" {
            self.pop_until_not_call();
        }
    }

    fn apply_app_state_route(&mut self, previous_app_state: &UiScreen, app_state: &UiScreen) {
        if app_state == previous_app_state {
            return;
        }
        if self.active_screen != *app_state {
            self.screen_stack.clear();
            self.active_screen = *app_state;
            self.focus_index = 0;
        }
    }

    fn advance_focus(&mut self) {
        let count = self.focus_count();
        self.focus_index = match screen_entry(self.active_screen).focus_policy {
            FocusPolicy::None => self.focus_index,
            FocusPolicy::Wrap => focus::advance(self.focus_index, count),
            FocusPolicy::Clamp => focus::advance_clamped(self.focus_index, count),
        };
    }

    fn select_focused(&mut self) {
        match self.active_screen {
            UiScreen::Hub => match self.focus_index {
                0 => self.push_screen(UiScreen::Listen),
                1 => self.push_screen(UiScreen::Talk),
                2 => self.push_screen(UiScreen::Ask),
                _ => self.push_screen(UiScreen::Power),
            },
            UiScreen::Listen => match self.focus_index {
                0 => self.push_screen(UiScreen::Playlists),
                1 => self.push_screen(UiScreen::RecentTracks),
                _ => {
                    self.intents.push(UiIntent::Music(MusicIntent::ShuffleAll));
                    self.push_screen(UiScreen::NowPlaying);
                }
            },
            UiScreen::Talk => match self.focus_index {
                0 => self.push_screen(UiScreen::Contacts),
                1 => self.push_screen(UiScreen::CallHistory),
                _ => self.push_screen(UiScreen::VoiceNote),
            },
            UiScreen::Playlists => {
                if let Some(item) = self.snapshot.music.playlists.get(self.focus_index).cloned() {
                    self.intents.push(UiIntent::Music(MusicIntent::LoadPlaylist(
                        intents::list_item_action(&item),
                    )));
                    self.push_screen(UiScreen::NowPlaying);
                }
            }
            UiScreen::RecentTracks => {
                if let Some(item) = self
                    .snapshot
                    .music
                    .recent_tracks
                    .get(self.focus_index)
                    .cloned()
                {
                    self.intents
                        .push(UiIntent::Music(MusicIntent::PlayRecentTrack(
                            intents::list_item_action(&item),
                        )));
                    self.push_screen(UiScreen::NowPlaying);
                }
            }
            UiScreen::NowPlaying => self.intents.push(UiIntent::Music(MusicIntent::PlayPause)),
            UiScreen::Ask => self.intents.push(UiIntent::Voice(VoiceIntent::AskStart)),
            UiScreen::VoiceNote => self.select_voice_note(),
            UiScreen::Contacts => {
                if let Some(item) = self.snapshot.call.contacts.get(self.focus_index).cloned() {
                    self.selected_contact = Some(item);
                    self.push_screen(UiScreen::TalkContact);
                }
            }
            UiScreen::TalkContact => self.select_talk_contact_action(),
            UiScreen::CallHistory => {
                if let Some(item) = self.snapshot.call.history.get(self.focus_index).cloned() {
                    self.emit_call_start(&item);
                }
            }
            UiScreen::IncomingCall => self.intents.push(UiIntent::Call(CallIntent::Answer)),
            UiScreen::InCall => self.intents.push(UiIntent::Call(CallIntent::ToggleMute)),
            UiScreen::Power => self.advance_focus(),
            _ => {}
        }
    }

    fn go_back_or_emit(&mut self) {
        if self.active_screen == UiScreen::VoiceNote && self.voice_note_phase() == "recording" {
            self.intents
                .push(UiIntent::Voice(VoiceIntent::CaptureCancel));
            self.pop_screen_or_hub();
            return;
        }
        if self.active_screen == UiScreen::VoiceNote
            && matches!(
                self.voice_note_phase().as_str(),
                "review" | "failed" | "sent"
            )
        {
            self.intents.push(UiIntent::Voice(VoiceIntent::Discard));
            self.pop_screen_or_hub();
            return;
        }

        match screen_entry(self.active_screen).navigation_policy {
            NavigationPolicy::Root => {}
            NavigationPolicy::Overlay | NavigationPolicy::Stack => self.pop_screen_or_hub(),
            NavigationPolicy::Call => self.go_back_from_call_screen(),
        }
    }

    fn go_back_from_call_screen(&mut self) {
        match self.active_screen {
            UiScreen::IncomingCall => self.intents.push(UiIntent::Call(CallIntent::Reject)),
            UiScreen::OutgoingCall | UiScreen::InCall => {
                self.intents.push(UiIntent::Call(CallIntent::Hangup));
            }
            _ => {}
        }
    }

    fn emit_call_start(&mut self, item: &ListItemSnapshot) {
        self.intents
            .push(UiIntent::Call(CallIntent::Start(intents::contact_action(
                item,
            ))));
    }

    fn select_talk_contact_action(&mut self) {
        let actions =
            screens::call::talk_contact_actions(&self.snapshot, self.selected_contact.as_ref());
        let Some(action) = actions.get(self.focus_index) else {
            return;
        };
        match action.kind {
            "call" => {
                if let Some(item) = self.selected_contact.clone() {
                    self.emit_call_start(&item);
                }
            }
            "voice_note" => self.push_screen(UiScreen::VoiceNote),
            "play_note" => {
                if let Some(payload) = self.latest_voice_note_payload() {
                    self.intents
                        .push(UiIntent::Voice(VoiceIntent::PlayLatest(payload)));
                }
            }
            _ => {}
        }
    }

    fn select_voice_note(&mut self) {
        match self.voice_note_phase().as_str() {
            "ready" => self.pop_screen_or_hub(),
            "recording" => self.intents.push(UiIntent::Voice(VoiceIntent::CaptureStop)),
            "review" => match self.focus_index {
                0 => {
                    if let Some(payload) = self.voice_note_recipient_payload() {
                        self.intents
                            .push(UiIntent::Voice(VoiceIntent::Send(payload)));
                    }
                }
                1 => self.intents.push(UiIntent::Voice(VoiceIntent::Play(None))),
                _ => self.intents.push(UiIntent::Voice(VoiceIntent::Discard)),
            },
            "failed" => match self.focus_index {
                0 => {
                    if let Some(payload) = self.voice_note_recipient_payload() {
                        self.intents
                            .push(UiIntent::Voice(VoiceIntent::Send(payload)));
                    }
                }
                _ => self.intents.push(UiIntent::Voice(VoiceIntent::Discard)),
            },
            "sent" => {
                self.intents.push(UiIntent::Voice(VoiceIntent::Discard));
                self.pop_screen_or_hub();
            }
            "sending" => {}
            _ => {}
        }
    }

    fn handle_ptt_press(&mut self) {
        if self.active_screen == UiScreen::VoiceNote && self.voice_note_phase() == "ready" {
            if let Some(payload) = self.voice_note_recipient_payload() {
                self.intents
                    .push(UiIntent::Voice(VoiceIntent::CaptureStart(payload)));
            }
            return;
        }
        if self.active_screen == UiScreen::Ask {
            self.intents.push(UiIntent::Voice(VoiceIntent::AskStart));
        }
    }

    fn handle_ptt_release(&mut self) {
        if self.active_screen == UiScreen::VoiceNote && self.voice_note_phase() == "recording" {
            self.intents.push(UiIntent::Voice(VoiceIntent::CaptureStop));
            return;
        }
        if self.active_screen == UiScreen::Ask {
            self.intents.push(UiIntent::Voice(VoiceIntent::AskStop));
        }
    }

    pub fn wants_ptt_passthrough(&self) -> bool {
        self.active_screen == UiScreen::VoiceNote
            && matches!(self.voice_note_phase().as_str(), "ready" | "recording")
    }

    fn voice_note_phase(&self) -> String {
        let phase = self.snapshot.voice.phase.trim().to_ascii_lowercase();
        if self.snapshot.voice.capture_in_flight
            || self.snapshot.voice.ptt_active
            || phase == "recording"
        {
            return "recording".to_string();
        }
        if matches!(phase.as_str(), "review" | "sending" | "sent" | "failed") {
            return phase;
        }
        "ready".to_string()
    }

    fn voice_note_recipient_payload(&self) -> Option<VoiceRecipientAction> {
        let contact = self
            .selected_contact
            .as_ref()
            .or_else(|| self.snapshot.call.contacts.first())?;
        intents::voice_recipient_action(contact)
    }

    fn latest_voice_note_payload(&self) -> Option<VoiceFileAction> {
        let contact = self
            .selected_contact
            .as_ref()
            .or_else(|| self.snapshot.call.contacts.first())?;
        let note = self
            .snapshot
            .call
            .latest_voice_note_by_contact
            .get(&contact.id)?;
        intents::voice_file_action(contact, note)
    }

    fn push_screen(&mut self, screen: UiScreen) {
        if self.active_screen != screen {
            self.screen_stack.push(self.active_screen);
        }
        self.active_screen = screen;
        self.focus_index = 0;
    }

    fn pop_screen_or_hub(&mut self) {
        self.active_screen = self.screen_stack.pop().unwrap_or(UiScreen::Hub);
        self.focus_index = 0;
    }

    fn pop_until_not_call(&mut self) {
        while navigator::is_call_screen(self.active_screen) {
            self.active_screen = self.screen_stack.pop().unwrap_or(UiScreen::Hub);
        }
        self.focus_index = 0;
    }

    fn pop_until_not_overlay(&mut self) {
        while navigator::is_overlay_screen(self.active_screen) {
            self.active_screen = self.screen_stack.pop().unwrap_or(UiScreen::Hub);
        }
        self.focus_index = 0;
    }

    fn clamp_focus(&mut self) {
        let count = self.focus_count();
        self.focus_index = focus::clamp(self.focus_index, count);
    }

    fn focus_count(&self) -> usize {
        focus::focus_count(
            self.active_screen,
            &self.snapshot,
            self.selected_contact.as_ref(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yoyopod_protocol::ui::{CallRuntimeSnapshot, MusicRuntimeSnapshot};

    #[test]
    fn runtime_patch_marks_only_changed_snapshot_domain() {
        let mut runtime = UiRuntime::default();
        runtime.mark_clean();

        runtime.apply_patch(RuntimeSnapshotPatch::Music(MusicRuntimeSnapshot {
            title: "Track".to_string(),
            ..MusicRuntimeSnapshot::default()
        }));

        let dirty = runtime.dirty_state();
        assert!(dirty.music);
        assert!(!dirty.call);
        assert!(!dirty.network);
        assert!(!dirty.full);
        assert!(!dirty.navigation);
    }

    #[test]
    fn runtime_preemption_marks_navigation_dirty() {
        let mut runtime = UiRuntime::default();
        runtime.mark_clean();

        runtime.apply_patch(RuntimeSnapshotPatch::Call(CallRuntimeSnapshot {
            state: "incoming".to_string(),
            peer_name: "Ada".to_string(),
            ..CallRuntimeSnapshot::default()
        }));

        let dirty = runtime.dirty_state();
        assert!(dirty.call);
        assert!(dirty.navigation);
        assert_eq!(runtime.active_screen(), UiScreen::IncomingCall);
    }

    #[test]
    fn animation_request_marks_runtime_dirty_until_complete() {
        let mut runtime = UiRuntime::default();
        runtime.mark_clean();

        runtime.start_animation(
            AnimationRequest {
                transition_id: "selection_move".to_string(),
                duration_ms: 200,
            },
            1000,
        );

        assert_eq!(runtime.active_transitions().len(), 1);
        assert!(runtime.dirty_state().animation);
        runtime.mark_clean();

        assert!(runtime.advance_animations(1100));
        assert_eq!(runtime.active_transitions().len(), 1);
        assert!(runtime.dirty_state().animation);
        runtime.mark_clean();

        assert!(runtime.advance_animations(1200));
        assert!(runtime.active_transitions().is_empty());
        assert!(runtime.dirty_state().animation);
        runtime.mark_clean();

        assert!(!runtime.advance_animations(1300));
        assert!(!runtime.is_dirty());
    }

    #[test]
    fn wrap_focus_policy_cycles_focus() {
        let mut runtime = UiRuntime {
            active_screen: UiScreen::Hub,
            focus_index: 3,
            ..UiRuntime::default()
        };

        runtime.handle_input(InputAction::Advance);

        assert_eq!(runtime.focus_index(), 0);
    }

    #[test]
    fn clamp_focus_policy_stops_at_last_item() {
        let mut runtime = UiRuntime::default();
        runtime.apply_snapshot(RuntimeSnapshot {
            app_state: UiScreen::Playlists,
            music: MusicRuntimeSnapshot {
                playlists: vec![
                    ListItemSnapshot::new("one", "One", "", "playlist"),
                    ListItemSnapshot::new("two", "Two", "", "playlist"),
                ],
                ..MusicRuntimeSnapshot::default()
            },
            ..RuntimeSnapshot::default()
        });
        runtime.focus_index = 1;

        runtime.handle_input(InputAction::Advance);

        assert_eq!(runtime.focus_index(), 1);
    }

    #[test]
    fn none_focus_policy_does_not_advance_focus() {
        let mut runtime = UiRuntime::default();
        runtime.apply_snapshot(RuntimeSnapshot {
            app_state: UiScreen::NowPlaying,
            ..RuntimeSnapshot::default()
        });

        runtime.handle_input(InputAction::Advance);

        assert_eq!(runtime.focus_index(), 0);
    }

    #[test]
    fn stack_navigation_policy_pops_on_back() {
        let mut runtime = UiRuntime {
            active_screen: UiScreen::Playlists,
            screen_stack: vec![UiScreen::Hub],
            ..UiRuntime::default()
        };

        runtime.handle_input(InputAction::Back);

        assert_eq!(runtime.active_screen(), UiScreen::Hub);
    }

    #[test]
    fn call_navigation_policy_emits_call_back_intent() {
        let mut runtime = UiRuntime {
            active_screen: UiScreen::IncomingCall,
            ..UiRuntime::default()
        };

        runtime.handle_input(InputAction::Back);

        assert_eq!(
            runtime.take_intents(),
            vec![UiIntent::Call(CallIntent::Reject)]
        );
    }
}
