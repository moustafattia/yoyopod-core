mod codec;
mod dispatcher;
mod handshake;
mod inbound;
mod outbound;

use std::io::{Read, Write};
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use anyhow::Result;
use yoyopod_protocol::ui::{UiError, UiErrorCode, UiEvent};

use crate::app::{RenderState, UiRuntime};
use crate::hardware::{ButtonDevice, DisplayDevice};
use crate::input::{ButtonTiming, OneButtonMachine};

const MANAGER_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(15);

pub fn run_worker<R, W, E, D, B>(
    input: R,
    output: &mut W,
    errors: &mut E,
    mut display: D,
    mut button: B,
) -> Result<()>
where
    R: Read + Send + 'static,
    W: Write,
    E: Write,
    D: DisplayDevice,
    B: ButtonDevice,
{
    let mut input_events = 0usize;
    let mut button_machine = OneButtonMachine::new(ButtonTiming::default());
    let mut ui_runtime = UiRuntime::default();
    let mut render_state = RenderState::open(display.width(), display.height())?;
    let mut shutdown_complete_emitted = false;

    handshake::emit_ready(output, display.width(), display.height())?;

    let lines = codec::spawn_line_reader(input);
    loop {
        let line = match lines.recv_timeout(MANAGER_HEARTBEAT_TIMEOUT) {
            Ok(line) => line?,
            Err(RecvTimeoutError::Timeout) => {
                handshake::emit_manager_timeout(output, errors, MANAGER_HEARTBEAT_TIMEOUT)?;
                break;
            }
            Err(RecvTimeoutError::Disconnected) => break,
        };
        if line.trim().is_empty() {
            continue;
        }

        let envelope = match codec::decode_envelope(&line) {
            Ok(envelope) => envelope,
            Err(err) => {
                writeln!(errors, "protocol decode error: {err}")?;
                outbound::emit_event(
                    output,
                    UiEvent::Error(UiError::new(UiErrorCode::DecodeError, err.to_string())),
                )?;
                continue;
            }
        };

        let command = match inbound::decode_command(envelope) {
            Ok(command) => command,
            Err(err) => {
                writeln!(errors, "UI command decode error: {err}")?;
                outbound::emit_event(
                    output,
                    UiEvent::Error(UiError::new(UiErrorCode::InvalidCommand, err.to_string())),
                )?;
                continue;
            }
        };

        let outcome = dispatcher::dispatch_command(command);
        if handle_app_event(
            outcome.event,
            output,
            &mut display,
            &mut button,
            &mut ui_runtime,
            &mut button_machine,
            &mut render_state,
            &mut input_events,
        )? {
            shutdown_complete_emitted = true;
            break;
        }
    }

    if !shutdown_complete_emitted {
        handshake::emit_shutdown_complete(output)?;
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_app_event<W, D, B>(
    event: dispatcher::AppEvent,
    output: &mut W,
    display: &mut D,
    button: &mut B,
    ui_runtime: &mut UiRuntime,
    button_machine: &mut OneButtonMachine,
    render_state: &mut RenderState,
    input_events: &mut usize,
) -> Result<bool>
where
    W: Write,
    D: DisplayDevice,
    B: ButtonDevice,
{
    match event {
        dispatcher::AppEvent::SetBacklight { brightness } => {
            display.set_backlight(brightness)?;
        }
        dispatcher::AppEvent::RuntimeSnapshot(snapshot) => {
            ui_runtime.apply_snapshot(snapshot);
        }
        dispatcher::AppEvent::RuntimePatch(patch) => {
            ui_runtime.apply_patch(patch);
        }
        dispatcher::AppEvent::InputAction(action) => {
            *input_events += 1;
            let now_ms = outbound::monotonic_millis();
            outbound::emit_input_action(output, action, "command", now_ms, 0)?;
            ui_runtime.handle_input(action);
            outbound::emit_intents(output, ui_runtime.take_intents())?;
        }
        dispatcher::AppEvent::Tick => {
            let now_ms = outbound::monotonic_millis();
            ui_runtime.advance_animations(now_ms);
            handle_button_input(
                output,
                button,
                button_machine,
                ui_runtime,
                input_events,
                now_ms,
            )?;
            outbound::emit_intents(output, ui_runtime.take_intents())?;
        }
        dispatcher::AppEvent::PollInput => {
            let now_ms = outbound::monotonic_millis();
            handle_button_input(
                output,
                button,
                button_machine,
                ui_runtime,
                input_events,
                now_ms,
            )?;
            outbound::emit_intents(output, ui_runtime.take_intents())?;
        }
        dispatcher::AppEvent::Health => {
            outbound::emit_event(output, ui_runtime.health_event(render_state, *input_events))?;
        }
        dispatcher::AppEvent::Animate(request) => {
            ui_runtime.start_animation(request, outbound::monotonic_millis());
        }
        dispatcher::AppEvent::Shutdown => {
            handshake::emit_shutdown_complete(output)?;
            return Ok(true);
        }
    }

    if let Some(event) =
        ui_runtime.render_if_dirty(display, render_state, outbound::monotonic_millis())?
    {
        outbound::emit_event(output, event)?;
    }
    Ok(false)
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
        outbound::emit_input_action(
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
