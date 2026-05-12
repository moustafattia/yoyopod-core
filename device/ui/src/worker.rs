use std::io::{BufRead, BufReader, Read, Write};

use anyhow::{bail, Result};
use serde_json::json;

use crate::framebuffer::Framebuffer;
use crate::hardware::{ButtonDevice, DisplayDevice};
use crate::input::{ButtonTiming, InputAction, OneButtonMachine};
use crate::protocol::{error_envelope, event_envelope, Envelope, EnvelopeKind};
use crate::render::LvglRenderer;
use crate::runtime::{RuntimeSnapshot, UiIntent, UiRuntime, UiScreen};
use crate::screens::ScreenModel;

pub fn run_worker<R, W, E, D, B>(
    input: R,
    output: &mut W,
    errors: &mut E,
    mut display: D,
    mut button: B,
) -> Result<()>
where
    R: Read,
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

    emit(
        output,
        event_envelope(
            "ui.ready",
            json!({
                "display": {"width": display.width(), "height": display.height()},
            }),
        ),
    )?;

    let reader = BufReader::new(input);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match Envelope::decode(line.as_bytes()) {
            Ok(envelope) => {
                if envelope.kind != EnvelopeKind::Command {
                    emit(
                        output,
                        error_envelope("invalid_kind", "worker accepts commands only"),
                    )?;
                    continue;
                }

                match envelope.message_type.as_str() {
                    "ui.set_backlight" => {
                        let brightness = envelope
                            .payload
                            .get("brightness")
                            .and_then(|value| value.as_f64())
                            .unwrap_or(0.8) as f32;
                        display.set_backlight(brightness.clamp(0.0, 1.0))?;
                    }
                    "ui.runtime_snapshot" => {
                        let snapshot = RuntimeSnapshot::from_payload(&envelope.payload)?;
                        ui_runtime.apply_snapshot(snapshot);
                        if render_runtime_if_dirty(
                            output,
                            &mut display,
                            &mut framebuffer,
                            &mut ui_runtime,
                            &mut last_active_screen,
                            &mut last_ui_renderer,
                            &mut lvgl_renderer,
                        )? {
                            frames += 1;
                        }
                    }
                    "ui.input_action" => {
                        let action = parse_input_action(&envelope.payload)?;
                        ui_runtime.handle_input(action);
                        emit_intents(output, ui_runtime.take_intents())?;
                        if render_runtime_if_dirty(
                            output,
                            &mut display,
                            &mut framebuffer,
                            &mut ui_runtime,
                            &mut last_active_screen,
                            &mut last_ui_renderer,
                            &mut lvgl_renderer,
                        )? {
                            frames += 1;
                        }
                    }
                    "ui.tick" => {
                        let pressed = button.pressed()?;
                        let now_ms = crate::protocol::monotonic_millis();
                        let button_events = if ui_runtime.wants_ptt_passthrough() {
                            button_machine.observe_ptt_passthrough(pressed, now_ms)
                        } else {
                            button_machine.observe(pressed, now_ms)
                        };
                        for event in button_events {
                            input_events += 1;
                            emit(
                                output,
                                event_envelope(
                                    "ui.input",
                                    json!({
                                        "action": event.action.as_str(),
                                        "method": event.method,
                                        "timestamp_ms": event.timestamp_ms,
                                        "duration_ms": event.duration_ms,
                                    }),
                                ),
                            )?;
                            ui_runtime.handle_input(event.action);
                        }
                        emit_intents(output, ui_runtime.take_intents())?;
                        if render_runtime_if_dirty(
                            output,
                            &mut display,
                            &mut framebuffer,
                            &mut ui_runtime,
                            &mut last_active_screen,
                            &mut last_ui_renderer,
                            &mut lvgl_renderer,
                        )? {
                            frames += 1;
                        }
                    }
                    "ui.poll_input" => {
                        let pressed = button.pressed()?;
                        let now_ms = crate::protocol::monotonic_millis();
                        let button_events = if ui_runtime.wants_ptt_passthrough() {
                            button_machine.observe_ptt_passthrough(pressed, now_ms)
                        } else {
                            button_machine.observe(pressed, now_ms)
                        };
                        for event in button_events {
                            input_events += 1;
                            emit(
                                output,
                                event_envelope(
                                    "ui.input",
                                    json!({
                                        "action": event.action.as_str(),
                                        "method": event.method,
                                        "timestamp_ms": event.timestamp_ms,
                                        "duration_ms": event.duration_ms,
                                    }),
                                ),
                            )?;
                        }
                    }
                    "ui.health" => {
                        let active_screen = ui_runtime.active_screen_model().screen();
                        emit(
                            output,
                            event_envelope(
                                "ui.health",
                                json!({
                                    "frames": frames,
                                    "button_events": input_events,
                                    "last_ui_renderer": last_ui_renderer,
                                    "active_screen": active_screen.as_str(),
                                }),
                            ),
                        )?;
                    }
                    "ui.shutdown" | "worker.stop" => break,
                    other => {
                        writeln!(errors, "unknown UI worker command: {other}")?;
                        emit(output, error_envelope("unknown_command", other))?;
                    }
                }
            }
            Err(err) => {
                writeln!(errors, "protocol decode error: {err}")?;
                emit(output, error_envelope("decode_error", err.to_string()))?;
            }
        }
    }

    Ok(())
}

fn emit<W: Write>(output: &mut W, envelope: Envelope) -> Result<()> {
    output.write_all(&envelope.encode()?)?;
    output.flush()?;
    Ok(())
}

fn emit_intents<W: Write>(output: &mut W, intents: Vec<UiIntent>) -> Result<()> {
    for intent in intents {
        emit(
            output,
            event_envelope(
                "ui.intent",
                json!({
                    "domain": intent.domain,
                    "action": intent.action,
                    "payload": intent.payload,
                }),
            ),
        )?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_runtime_if_dirty<W, D>(
    output: &mut W,
    display: &mut D,
    framebuffer: &mut Framebuffer,
    ui_runtime: &mut UiRuntime,
    last_active_screen: &mut Option<UiScreen>,
    last_ui_renderer: &mut String,
    lvgl_renderer: &mut LvglRenderer,
) -> Result<bool>
where
    W: Write,
    D: DisplayDevice,
{
    if !ui_runtime.is_dirty() {
        return Ok(false);
    }

    let screen_model = ui_runtime.active_screen_model();
    lvgl_renderer.render_screen_model(framebuffer, &screen_model)?;
    *last_ui_renderer = "lvgl".to_string();
    display.flush_full_frame(framebuffer)?;
    emit_screen_changed_if_needed(output, last_active_screen, &screen_model)?;
    ui_runtime.mark_clean();
    Ok(true)
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
        emit(
            output,
            event_envelope(
                "ui.screen_changed",
                json!({
                    "screen": screen_model.screen().as_str(),
                    "title": screen_model_title(screen_model),
                }),
            ),
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

fn parse_input_action(payload: &serde_json::Value) -> Result<InputAction> {
    let action = payload
        .get("action")
        .and_then(|value| value.as_str())
        .unwrap_or("");
    match action {
        "advance" => Ok(InputAction::Advance),
        "select" => Ok(InputAction::Select),
        "back" => Ok(InputAction::Back),
        "ptt_press" => Ok(InputAction::PttPress),
        "ptt_release" => Ok(InputAction::PttRelease),
        value => bail!("unknown UI input action {value:?}"),
    }
}
