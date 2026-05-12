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

use crate::app::{UiRuntime, UiScreen};
use crate::hardware::{ButtonDevice, DisplayDevice};
use crate::input::{ButtonTiming, OneButtonMachine};
use crate::render::{Framebuffer, LvglRenderer};

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
    let mut framebuffer = Framebuffer::new(display.width(), display.height());
    let mut frames = 0usize;
    let mut input_events = 0usize;
    let mut last_ui_renderer = String::new();
    let mut button_machine = OneButtonMachine::new(ButtonTiming::default());
    let mut ui_runtime = UiRuntime::default();
    let mut last_active_screen: Option<UiScreen> = None;
    let mut lvgl_renderer = LvglRenderer::open(None)?;
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

        if dispatcher::dispatch_command(
            command,
            output,
            &mut display,
            &mut button,
            &mut framebuffer,
            &mut ui_runtime,
            &mut button_machine,
            &mut last_active_screen,
            &mut last_ui_renderer,
            &mut lvgl_renderer,
            &mut frames,
            &mut input_events,
        )? == dispatcher::DispatchOutcome::Shutdown
        {
            shutdown_complete_emitted = true;
            break;
        }
    }

    if !shutdown_complete_emitted {
        handshake::emit_shutdown_complete(output)?;
    }

    Ok(())
}
