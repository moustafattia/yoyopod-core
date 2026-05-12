use std::io::Write;

use anyhow::Result;
use yoyopod_protocol::ui::{UiCommand, UiEvent, UiHealth, UiScreenChanged};

use crate::app::{UiRuntime, UiScreen};
use crate::hardware::{ButtonDevice, DisplayDevice};
use crate::input::OneButtonMachine;
use crate::presentation::screens::ScreenModel;
use crate::render::{Framebuffer, LvglRenderer};

use super::outbound::{emit_event, emit_input_action, emit_intents, monotonic_millis};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DispatchOutcome {
    Continue,
    Shutdown,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch_command<W, D, B>(
    command: UiCommand,
    output: &mut W,
    display: &mut D,
    button: &mut B,
    framebuffer: &mut Framebuffer,
    ui_runtime: &mut UiRuntime,
    button_machine: &mut OneButtonMachine,
    last_active_screen: &mut Option<UiScreen>,
    last_ui_renderer: &mut String,
    lvgl_renderer: &mut LvglRenderer,
    frames: &mut usize,
    input_events: &mut usize,
) -> Result<DispatchOutcome>
where
    W: Write,
    D: DisplayDevice,
    B: ButtonDevice,
{
    match command {
        UiCommand::SetBacklight { brightness } => {
            display.set_backlight(brightness)?;
        }
        UiCommand::RuntimeSnapshot(snapshot) => {
            ui_runtime.apply_snapshot(snapshot);
            render_if_dirty(
                output,
                display,
                framebuffer,
                ui_runtime,
                last_active_screen,
                last_ui_renderer,
                lvgl_renderer,
                frames,
            )?;
        }
        UiCommand::RuntimePatch(patch) => {
            ui_runtime.apply_patch(patch);
            render_if_dirty(
                output,
                display,
                framebuffer,
                ui_runtime,
                last_active_screen,
                last_ui_renderer,
                lvgl_renderer,
                frames,
            )?;
        }
        UiCommand::InputAction(action) => {
            *input_events += 1;
            emit_input_action(output, action, "command", monotonic_millis(), 0)?;
            ui_runtime.handle_input(action);
            emit_intents(output, ui_runtime.take_intents())?;
            render_if_dirty(
                output,
                display,
                framebuffer,
                ui_runtime,
                last_active_screen,
                last_ui_renderer,
                lvgl_renderer,
                frames,
            )?;
        }
        UiCommand::Tick => {
            let now_ms = monotonic_millis();
            ui_runtime.advance_animations(now_ms);
            handle_button_input(
                output,
                button,
                button_machine,
                ui_runtime,
                input_events,
                now_ms,
            )?;
            emit_intents(output, ui_runtime.take_intents())?;
            render_if_dirty(
                output,
                display,
                framebuffer,
                ui_runtime,
                last_active_screen,
                last_ui_renderer,
                lvgl_renderer,
                frames,
            )?;
        }
        UiCommand::PollInput => {
            let now_ms = monotonic_millis();
            handle_button_input(
                output,
                button,
                button_machine,
                ui_runtime,
                input_events,
                now_ms,
            )?;
            emit_intents(output, ui_runtime.take_intents())?;
            render_if_dirty(
                output,
                display,
                framebuffer,
                ui_runtime,
                last_active_screen,
                last_ui_renderer,
                lvgl_renderer,
                frames,
            )?;
        }
        UiCommand::Health => {
            let active_screen = ui_runtime.active_screen_model().screen();
            emit_event(
                output,
                UiEvent::Health(UiHealth {
                    frames: *frames,
                    button_events: *input_events,
                    last_ui_renderer: last_ui_renderer.clone(),
                    active_screen,
                }),
            )?;
        }
        UiCommand::Animate(request) => {
            ui_runtime.start_animation(request, monotonic_millis());
            render_if_dirty(
                output,
                display,
                framebuffer,
                ui_runtime,
                last_active_screen,
                last_ui_renderer,
                lvgl_renderer,
                frames,
            )?;
        }
        UiCommand::Shutdown | UiCommand::WorkerStop => {
            super::handshake::emit_shutdown_complete(output)?;
            return Ok(DispatchOutcome::Shutdown);
        }
    }

    Ok(DispatchOutcome::Continue)
}

fn handle_button_input<W, B>(
    output: &mut W,
    button: &mut B,
    button_machine: &mut OneButtonMachine,
    ui_runtime: &mut UiRuntime,
    input_events: &mut usize,
    now_ms: u64,
) -> Result<()>
where
    W: Write,
    B: ButtonDevice,
{
    let pressed = button.pressed()?;
    let button_events = if ui_runtime.wants_ptt_passthrough() {
        button_machine.observe_ptt_passthrough(pressed, now_ms)
    } else {
        button_machine.observe(pressed, now_ms)
    };
    for event in button_events {
        *input_events += 1;
        emit_input_action(
            output,
            event.action,
            event.method,
            event.timestamp_ms,
            event.duration_ms,
        )?;
        ui_runtime.handle_input(event.action);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_if_dirty<W, D>(
    output: &mut W,
    display: &mut D,
    framebuffer: &mut Framebuffer,
    ui_runtime: &mut UiRuntime,
    last_active_screen: &mut Option<UiScreen>,
    last_ui_renderer: &mut String,
    lvgl_renderer: &mut LvglRenderer,
    frames: &mut usize,
) -> Result<()>
where
    W: Write,
    D: DisplayDevice,
{
    if !ui_runtime.is_dirty() {
        return Ok(());
    }

    let screen_model = ui_runtime.active_screen_model();
    lvgl_renderer.render_screen_model(framebuffer, &screen_model)?;
    *last_ui_renderer = "lvgl".to_string();
    display.flush_full_frame(framebuffer)?;
    emit_screen_changed_if_needed(output, last_active_screen, &screen_model)?;
    ui_runtime.mark_clean();
    *frames += 1;
    Ok(())
}

fn emit_screen_changed_if_needed<W: Write>(
    output: &mut W,
    last_active_screen: &mut Option<UiScreen>,
    screen_model: &ScreenModel,
) -> Result<()> {
    if last_active_screen
        .map(|screen| screen != screen_model.screen())
        .unwrap_or(true)
    {
        emit_event(
            output,
            UiEvent::ScreenChanged(UiScreenChanged {
                screen: screen_model.screen(),
                title: screen_model_title(screen_model).to_string(),
            }),
        )?;
        *last_active_screen = Some(screen_model.screen());
    }
    Ok(())
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
