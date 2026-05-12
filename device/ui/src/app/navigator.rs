use crate::presentation::registry::{
    screen_entry, static_intent_template, BackPolicy, DynamicActionKind, FocusPolicy,
    IntentTemplate, ListKind, NavigationPolicy, PassthroughPolicy, SelectionTarget,
    SnapshotCondition,
};
use crate::presentation::screens;
use yoyopod_protocol::ui::{
    CallIntent, ListItemSnapshot, MusicIntent, RuntimeSnapshot, UiIntent, VoiceIntent,
};

use super::{focus, intents, UiRuntime, UiScreen};

pub fn runtime_preemption(snapshot: &RuntimeSnapshot) -> Option<UiScreen> {
    if !snapshot.overlay.error.trim().is_empty() {
        return Some(UiScreen::Error);
    }
    if snapshot.overlay.loading {
        return Some(UiScreen::Loading);
    }
    match snapshot.call.state.as_str() {
        "incoming" => Some(UiScreen::IncomingCall),
        "outgoing" => Some(UiScreen::OutgoingCall),
        "active" => Some(UiScreen::InCall),
        _ => None,
    }
}

pub fn apply_runtime_preemption(runtime: &mut UiRuntime) {
    if let Some(screen) = runtime_preemption(&runtime.snapshot) {
        if runtime.active_screen != screen {
            push_screen(runtime, screen);
        }
        return;
    }

    if is_overlay_screen(runtime.active_screen) {
        pop_until_not_overlay(runtime);
    }

    if is_call_screen(runtime.active_screen) && runtime.snapshot.call.state == "idle" {
        pop_until_not_call(runtime);
    }
}

pub fn apply_app_state_route(
    runtime: &mut UiRuntime,
    previous_app_state: &UiScreen,
    app_state: &UiScreen,
) {
    if app_state == previous_app_state {
        return;
    }
    if runtime.active_screen != *app_state {
        runtime.screen_stack.clear();
        runtime.active_screen = *app_state;
        runtime.focus_index = 0;
    }
}

pub fn advance_focus(runtime: &mut UiRuntime) {
    let count = focus_count(runtime);
    runtime.focus_index = match screen_entry(runtime.active_screen).focus_policy {
        FocusPolicy::None => runtime.focus_index,
        FocusPolicy::Wrap => focus::advance(runtime.focus_index, count),
        FocusPolicy::Clamp => focus::advance_clamped(runtime.focus_index, count),
    };
}

pub fn select_focused(runtime: &mut UiRuntime) {
    let targets = screen_entry(runtime.active_screen).select_targets;
    let Some(target) = targets
        .get(runtime.focus_index)
        .or_else(|| targets.last())
        .copied()
    else {
        return;
    };
    apply_selection_target(runtime, target);
}

pub fn go_back_or_emit(runtime: &mut UiRuntime) {
    if apply_back_passthrough(runtime) {
        return;
    }

    match screen_entry(runtime.active_screen).navigation_policy {
        NavigationPolicy::Root => {}
        NavigationPolicy::Overlay | NavigationPolicy::Stack => pop_screen_or_hub(runtime),
        NavigationPolicy::Call => go_back_from_call_screen(runtime),
    }
}

pub fn handle_ptt_press(runtime: &mut UiRuntime) {
    apply_passthrough_trigger(runtime, yoyopod_protocol::ui::InputAction::PttPress);
}

pub fn handle_ptt_release(runtime: &mut UiRuntime) {
    apply_passthrough_trigger(runtime, yoyopod_protocol::ui::InputAction::PttRelease);
}

pub fn wants_ptt_passthrough(runtime: &UiRuntime) -> bool {
    screen_entry(runtime.active_screen)
        .passthrough_policies
        .iter()
        .any(|policy| policy.captures_button && matches_condition(runtime, policy.when))
}

pub fn clamp_focus(runtime: &mut UiRuntime) {
    let count = focus_count(runtime);
    runtime.focus_index = focus::clamp(runtime.focus_index, count);
}

pub fn is_call_screen(screen: UiScreen) -> bool {
    matches!(
        screen,
        UiScreen::IncomingCall | UiScreen::OutgoingCall | UiScreen::InCall
    )
}

pub fn is_overlay_screen(screen: UiScreen) -> bool {
    matches!(screen, UiScreen::Loading | UiScreen::Error)
}

fn apply_selection_target(runtime: &mut UiRuntime, target: SelectionTarget) {
    match target {
        SelectionTarget::PushScreen(screen) => push_screen(runtime, screen),
        SelectionTarget::EmitIntent(template) => emit_static_intent(runtime, template),
        SelectionTarget::PushWithIntent { screen, intent } => {
            emit_static_intent(runtime, intent);
            push_screen(runtime, screen);
        }
        SelectionTarget::DynamicListItem { kind } => select_dynamic_list_item(runtime, kind),
        SelectionTarget::DynamicAction { kind } => select_dynamic_action(runtime, kind),
        SelectionTarget::AdvanceFocus => advance_focus(runtime),
        SelectionTarget::Noop => {}
    }
}

fn emit_static_intent(runtime: &mut UiRuntime, template: IntentTemplate) {
    if let Some(intent) = static_intent_template(template) {
        runtime.intents.push(intent);
    }
}

fn select_dynamic_list_item(runtime: &mut UiRuntime, kind: ListKind) {
    match kind {
        ListKind::Playlists => {
            if let Some(item) = runtime
                .snapshot
                .music
                .playlists
                .get(runtime.focus_index)
                .cloned()
            {
                runtime
                    .intents
                    .push(UiIntent::Music(MusicIntent::LoadPlaylist(
                        intents::list_item_action(&item),
                    )));
                push_screen(runtime, UiScreen::NowPlaying);
            }
        }
        ListKind::RecentTracks => {
            if let Some(item) = runtime
                .snapshot
                .music
                .recent_tracks
                .get(runtime.focus_index)
                .cloned()
            {
                runtime
                    .intents
                    .push(UiIntent::Music(MusicIntent::PlayRecentTrack(
                        intents::list_item_action(&item),
                    )));
                push_screen(runtime, UiScreen::NowPlaying);
            }
        }
        ListKind::Contacts => {
            if let Some(item) = runtime
                .snapshot
                .call
                .contacts
                .get(runtime.focus_index)
                .cloned()
            {
                runtime.selected_contact = Some(item);
                push_screen(runtime, UiScreen::TalkContact);
            }
        }
        ListKind::CallHistory => {
            if let Some(item) = runtime
                .snapshot
                .call
                .history
                .get(runtime.focus_index)
                .cloned()
            {
                emit_call_start(runtime, &item);
            }
        }
    }
}

fn select_dynamic_action(runtime: &mut UiRuntime, kind: DynamicActionKind) {
    match kind {
        DynamicActionKind::TalkContact => select_talk_contact_action(runtime),
        DynamicActionKind::VoiceNote => select_voice_note(runtime),
    }
}

fn apply_back_passthrough(runtime: &mut UiRuntime) -> bool {
    let policy = screen_entry(runtime.active_screen)
        .back_policies
        .iter()
        .find(|policy| matches_condition(runtime, policy.when))
        .copied();
    if let Some(policy) = policy {
        emit_back_intent(runtime, policy);
        if policy.pop_screen {
            pop_screen_or_hub(runtime);
        }
        return true;
    }
    false
}

fn emit_back_intent(runtime: &mut UiRuntime, policy: BackPolicy) {
    emit_static_intent(runtime, policy.intent);
}

fn go_back_from_call_screen(runtime: &mut UiRuntime) {
    match runtime.active_screen {
        UiScreen::IncomingCall => runtime.intents.push(UiIntent::Call(CallIntent::Reject)),
        UiScreen::OutgoingCall | UiScreen::InCall => {
            runtime.intents.push(UiIntent::Call(CallIntent::Hangup));
        }
        _ => {}
    }
}

fn emit_call_start(runtime: &mut UiRuntime, item: &ListItemSnapshot) {
    runtime
        .intents
        .push(UiIntent::Call(CallIntent::Start(intents::contact_action(
            item,
        ))));
}

fn select_talk_contact_action(runtime: &mut UiRuntime) {
    let actions =
        screens::call::talk_contact_actions(&runtime.snapshot, runtime.selected_contact.as_ref());
    let Some(action) = actions.get(runtime.focus_index) else {
        return;
    };
    match action.kind {
        "call" => {
            if let Some(item) = runtime.selected_contact.clone() {
                emit_call_start(runtime, &item);
            }
        }
        "voice_note" => push_screen(runtime, UiScreen::VoiceNote),
        "play_note" => {
            if let Some(payload) = runtime.latest_voice_note_payload() {
                runtime
                    .intents
                    .push(UiIntent::Voice(VoiceIntent::PlayLatest(payload)));
            }
        }
        _ => {}
    }
}

fn select_voice_note(runtime: &mut UiRuntime) {
    match runtime.voice_note_phase().as_str() {
        "ready" => pop_screen_or_hub(runtime),
        "recording" => runtime
            .intents
            .push(UiIntent::Voice(VoiceIntent::CaptureStop)),
        "review" => match runtime.focus_index {
            0 => {
                if let Some(payload) = runtime.voice_note_recipient_payload() {
                    runtime
                        .intents
                        .push(UiIntent::Voice(VoiceIntent::Send(payload)));
                }
            }
            1 => runtime
                .intents
                .push(UiIntent::Voice(VoiceIntent::Play(None))),
            _ => runtime.intents.push(UiIntent::Voice(VoiceIntent::Discard)),
        },
        "failed" => match runtime.focus_index {
            0 => {
                if let Some(payload) = runtime.voice_note_recipient_payload() {
                    runtime
                        .intents
                        .push(UiIntent::Voice(VoiceIntent::Send(payload)));
                }
            }
            _ => runtime.intents.push(UiIntent::Voice(VoiceIntent::Discard)),
        },
        "sent" => {
            runtime.intents.push(UiIntent::Voice(VoiceIntent::Discard));
            pop_screen_or_hub(runtime);
        }
        "sending" => {}
        _ => {}
    }
}

fn apply_passthrough_trigger(runtime: &mut UiRuntime, trigger: yoyopod_protocol::ui::InputAction) {
    let policy = screen_entry(runtime.active_screen)
        .passthrough_policies
        .iter()
        .find(|policy| policy.trigger == trigger && matches_condition(runtime, policy.when))
        .copied();
    if let Some(policy) = policy {
        emit_passthrough_intent(runtime, policy);
    }
}

fn emit_passthrough_intent(runtime: &mut UiRuntime, policy: PassthroughPolicy) {
    match policy.intent {
        IntentTemplate::VoiceCaptureStartRecipient => {
            if let Some(payload) = runtime.voice_note_recipient_payload() {
                runtime
                    .intents
                    .push(UiIntent::Voice(VoiceIntent::CaptureStart(payload)));
            }
        }
        template => emit_static_intent(runtime, template),
    }
}

fn matches_condition(runtime: &UiRuntime, condition: SnapshotCondition) -> bool {
    match condition {
        SnapshotCondition::Always => true,
        SnapshotCondition::VoiceReady => runtime.voice_note_phase() == "ready",
        SnapshotCondition::VoiceRecording => runtime.voice_note_phase() == "recording",
        SnapshotCondition::VoiceReviewOrFailedOrSent => matches!(
            runtime.voice_note_phase().as_str(),
            "review" | "failed" | "sent"
        ),
        SnapshotCondition::VoiceReadyOrRecording => {
            matches!(runtime.voice_note_phase().as_str(), "ready" | "recording")
        }
    }
}

fn push_screen(runtime: &mut UiRuntime, screen: UiScreen) {
    if runtime.active_screen != screen {
        runtime.screen_stack.push(runtime.active_screen);
    }
    runtime.active_screen = screen;
    runtime.focus_index = 0;
}

fn pop_screen_or_hub(runtime: &mut UiRuntime) {
    runtime.active_screen = runtime.screen_stack.pop().unwrap_or(UiScreen::Hub);
    runtime.focus_index = 0;
}

fn pop_until_not_call(runtime: &mut UiRuntime) {
    while is_call_screen(runtime.active_screen) {
        runtime.active_screen = runtime.screen_stack.pop().unwrap_or(UiScreen::Hub);
    }
    runtime.focus_index = 0;
}

fn pop_until_not_overlay(runtime: &mut UiRuntime) {
    while is_overlay_screen(runtime.active_screen) {
        runtime.active_screen = runtime.screen_stack.pop().unwrap_or(UiScreen::Hub);
    }
    runtime.focus_index = 0;
}

fn focus_count(runtime: &UiRuntime) -> usize {
    focus::focus_count(
        runtime.active_screen,
        &runtime.snapshot,
        runtime.selected_contact.as_ref(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use yoyopod_protocol::ui::{CallRuntimeSnapshot, InputAction, MusicRuntimeSnapshot};

    #[test]
    fn runtime_preemption_marks_navigation_dirty() {
        let mut runtime = UiRuntime::default();
        runtime.mark_clean();

        runtime.apply_patch(yoyopod_protocol::ui::RuntimeSnapshotPatch::Call(
            CallRuntimeSnapshot {
                state: "incoming".to_string(),
                peer_name: "Ada".to_string(),
                ..CallRuntimeSnapshot::default()
            },
        ));

        let dirty = runtime.dirty_state();
        assert!(dirty.call);
        assert!(dirty.navigation);
        assert_eq!(runtime.active_screen(), UiScreen::IncomingCall);
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
    fn stack_navigation_policy_pops_on_back() {
        let mut runtime = UiRuntime {
            active_screen: UiScreen::Playlists,
            screen_stack: vec![UiScreen::Hub],
            ..UiRuntime::default()
        };

        runtime.handle_input(InputAction::Back);

        assert_eq!(runtime.active_screen(), UiScreen::Hub);
    }
}
