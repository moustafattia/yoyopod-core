use std::io::{BufRead, BufReader, Read, Write};

use anyhow::Result;
use yoyopod_protocol::ui::{
    DisplayInfo, UiCommand, UiError, UiErrorCode, UiEvent, UiHealth, UiInputEvent, UiIntent,
    UiReady, UiScreenChanged,
};
use yoyopod_protocol::WorkerEnvelope;

use crate::app::{UiRuntime, UiScreen};
use crate::framebuffer::Framebuffer;
use crate::hardware::{ButtonDevice, DisplayDevice};
use crate::input::{ButtonTiming, OneButtonMachine};
use crate::presentation::screens::ScreenModel;
use crate::render::LvglRenderer;

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
    let mut shutdown_complete_emitted = false;

    emit_event(
        output,
        UiEvent::Ready(UiReady {
            display: DisplayInfo {
                width: display.width(),
                height: display.height(),
            },
        }),
    )?;

    let reader = BufReader::new(input);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        match WorkerEnvelope::decode(line.as_bytes()) {
            Ok(envelope) => {
                let command = match UiCommand::from_envelope(envelope) {
                    Ok(command) => command,
                    Err(err) => {
                        writeln!(errors, "UI command decode error: {err}")?;
                        emit_event(
                            output,
                            UiEvent::Error(UiError::new(
                                UiErrorCode::InvalidCommand,
                                err.to_string(),
                            )),
                        )?;
                        continue;
                    }
                };

                match command {
                    UiCommand::SetBacklight { brightness } => {
                        display.set_backlight(brightness)?;
                    }
                    UiCommand::RuntimeSnapshot(snapshot) => {
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
                    UiCommand::RuntimePatch(patch) => {
                        ui_runtime.apply_patch(patch);
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
                    UiCommand::InputAction(action) => {
                        input_events += 1;
                        emit_input_action(output, action, "command", monotonic_millis(), 0)?;
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
                    UiCommand::Tick => {
                        let pressed = button.pressed()?;
                        let now_ms = monotonic_millis();
                        ui_runtime.advance_animations(now_ms);
                        let button_events = if ui_runtime.wants_ptt_passthrough() {
                            button_machine.observe_ptt_passthrough(pressed, now_ms)
                        } else {
                            button_machine.observe(pressed, now_ms)
                        };
                        for event in button_events {
                            input_events += 1;
                            emit_input_action(
                                output,
                                event.action,
                                event.method,
                                event.timestamp_ms,
                                event.duration_ms,
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
                    UiCommand::PollInput => {
                        let pressed = button.pressed()?;
                        let now_ms = monotonic_millis();
                        let button_events = if ui_runtime.wants_ptt_passthrough() {
                            button_machine.observe_ptt_passthrough(pressed, now_ms)
                        } else {
                            button_machine.observe(pressed, now_ms)
                        };
                        for event in button_events {
                            input_events += 1;
                            emit_input_action(
                                output,
                                event.action,
                                event.method,
                                event.timestamp_ms,
                                event.duration_ms,
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
                    UiCommand::Health => {
                        let active_screen = ui_runtime.active_screen_model().screen();
                        emit_event(
                            output,
                            UiEvent::Health(UiHealth {
                                frames,
                                button_events: input_events,
                                last_ui_renderer: last_ui_renderer.clone(),
                                active_screen: active_screen.as_str().to_string(),
                            }),
                        )?;
                    }
                    UiCommand::Animate(request) => {
                        ui_runtime.start_animation(request, monotonic_millis());
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
                    UiCommand::Shutdown | UiCommand::WorkerStop => {
                        emit_event(output, UiEvent::ShutdownComplete)?;
                        shutdown_complete_emitted = true;
                        break;
                    }
                }
            }
            Err(err) => {
                writeln!(errors, "protocol decode error: {err}")?;
                emit_event(
                    output,
                    UiEvent::Error(UiError::new(UiErrorCode::DecodeError, err.to_string())),
                )?;
            }
        }
    }

    if !shutdown_complete_emitted {
        emit_event(output, UiEvent::ShutdownComplete)?;
    }

    Ok(())
}

fn emit<W: Write>(output: &mut W, envelope: WorkerEnvelope) -> Result<()> {
    output.write_all(&envelope.encode()?)?;
    output.flush()?;
    Ok(())
}

fn emit_event<W: Write>(output: &mut W, event: UiEvent) -> Result<()> {
    let mut envelope = event.into_envelope();
    envelope.timestamp_ms = monotonic_millis();
    emit(output, envelope)
}

fn emit_intents<W: Write>(output: &mut W, intents: Vec<UiIntent>) -> Result<()> {
    for intent in intents {
        emit_event(output, UiEvent::Intent(intent))?;
    }
    Ok(())
}

fn emit_input_action<W: Write>(
    output: &mut W,
    action: yoyopod_protocol::ui::InputAction,
    method: impl Into<String>,
    timestamp_ms: u64,
    duration_ms: u64,
) -> Result<()> {
    emit_event(
        output,
        UiEvent::Input(UiInputEvent {
            action,
            method: method.into(),
            timestamp_ms,
            duration_ms,
        }),
    )
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
        emit_event(
            output,
            UiEvent::ScreenChanged(UiScreenChanged {
                screen: screen_model.screen().as_str().to_string(),
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

fn monotonic_millis() -> u64 {
    use std::sync::OnceLock;
    use std::time::Instant;

    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_millis() as u64
}
