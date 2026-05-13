use std::io::{Read, Write};
use std::sync::mpsc::RecvTimeoutError;
use std::time::{Duration, Instant};

use anyhow::Result;
use yoyopod_protocol::ui::{UiError, UiErrorCode, UiEvent, UiHealth, UiScreen, UiScreenChanged};

use crate::application::UiRuntime;
use crate::components;
use crate::engine::Engine;
use crate::hardware::{ButtonDevice, DisplayDevice};
use crate::input::{ButtonTiming, OneButtonMachine};
use crate::renderer::{Framebuffer, LvglRenderer, RenderMode, Renderer};
use crate::router;
use crate::scene::{load_scene_defaults, GlobalClock, HudScene, SceneGraph};
use crate::transport::{codec, dispatcher, handshake, inbound, outbound};

const MANAGER_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(15);
const RUNTIME_TICK_TIMEOUT: Duration = Duration::from_secs(5);
const INPUT_POLL_INTERVAL: Duration = Duration::from_millis(250);

pub struct RenderState {
    framebuffer: Framebuffer,
    engine: Engine,
    renderer: Box<LvglRenderer>,
    frames: usize,
    last_active_screen: Option<UiScreen>,
    last_ui_renderer: String,
}

impl RenderState {
    pub fn open(width: usize, height: usize) -> Result<Self> {
        load_scene_defaults()?;
        let framebuffer = Framebuffer::new(width, height);
        let mut renderer = Box::new(LvglRenderer::open(None)?);
        renderer.initialize_display(&framebuffer)?;
        Ok(Self {
            framebuffer,
            engine: Engine::default(),
            renderer,
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
    let mut watchdog = RuntimeWatchdog::new();

    handshake::emit_ready(output, display.width(), display.height())?;

    let lines = codec::spawn_line_reader(input);
    loop {
        let line = match lines.recv_timeout(INPUT_POLL_INTERVAL) {
            Ok(line) => {
                watchdog.note_manager_message();
                line?
            }
            Err(RecvTimeoutError::Timeout) => {
                if watchdog.manager_timed_out() {
                    handshake::emit_manager_timeout(output, errors, MANAGER_HEARTBEAT_TIMEOUT)?;
                    break;
                }
                emit_runtime_stalled_if_needed(
                    output,
                    &mut display,
                    &mut ui_runtime,
                    &mut render_state,
                    &mut watchdog,
                )?;
                continue;
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
        if matches!(outcome.event, dispatcher::AppEvent::Tick) {
            watchdog.note_tick();
        }
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
        emit_runtime_stalled_if_needed(
            output,
            &mut display,
            &mut ui_runtime,
            &mut render_state,
            &mut watchdog,
        )?;
    }

    if !shutdown_complete_emitted {
        handshake::emit_shutdown_complete(output)?;
    }

    Ok(())
}

struct RuntimeWatchdog {
    last_manager_message: Instant,
    last_tick: Instant,
    runtime_stalled_emitted: bool,
}

impl RuntimeWatchdog {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            last_manager_message: now,
            last_tick: now,
            runtime_stalled_emitted: false,
        }
    }

    fn note_manager_message(&mut self) {
        self.last_manager_message = Instant::now();
    }

    fn note_tick(&mut self) {
        self.last_tick = Instant::now();
        self.runtime_stalled_emitted = false;
    }

    fn manager_timed_out(&self) -> bool {
        self.last_manager_message.elapsed() >= MANAGER_HEARTBEAT_TIMEOUT
    }

    fn runtime_stalled(&self) -> bool {
        self.last_tick.elapsed() >= RUNTIME_TICK_TIMEOUT && !self.runtime_stalled_emitted
    }
}

fn emit_runtime_stalled_if_needed<W, D>(
    output: &mut W,
    display: &mut D,
    ui_runtime: &mut UiRuntime,
    render_state: &mut RenderState,
    watchdog: &mut RuntimeWatchdog,
) -> Result<()>
where
    W: Write,
    D: DisplayDevice,
{
    if !watchdog.runtime_stalled() {
        return Ok(());
    }
    watchdog.runtime_stalled_emitted = true;
    ui_runtime.mark_runtime_stalled();
    outbound::emit_event(
        output,
        UiEvent::Error(UiError::new(
            UiErrorCode::RuntimeStalled,
            "runtime tick stalled",
        )),
    )?;
    if let Some(event) = render_if_dirty(
        ui_runtime,
        display,
        render_state,
        outbound::monotonic_millis(),
    )? {
        outbound::emit_event(output, event)?;
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
            if render_state.engine.animation_frame_dirty(now_ms) {
                ui_runtime.mark_animation_frame();
            }
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
            outbound::emit_event(
                output,
                health_event(ui_runtime, render_state, *input_events),
            )?;
        }
        dispatcher::AppEvent::Animate(request) => {
            ui_runtime.start_animation(request, outbound::monotonic_millis());
        }
        dispatcher::AppEvent::Shutdown => {
            handshake::emit_shutdown_complete(output)?;
            return Ok(true);
        }
    }

    if let Some(event) = render_if_dirty(
        ui_runtime,
        display,
        render_state,
        outbound::monotonic_millis(),
    )? {
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

fn render_if_dirty<D>(
    ui_runtime: &mut UiRuntime,
    display: &mut D,
    render: &mut RenderState,
    now_ms: u64,
) -> Result<Option<UiEvent>>
where
    D: DisplayDevice,
{
    if !ui_runtime.is_dirty() {
        return Ok(None);
    }

    let scene_graph = active_scene_graph(ui_runtime, now_ms);
    render.engine.tick_clocks(now_ms);
    let mutations = render.engine.render(&scene_graph, now_ms);
    render.renderer.apply(mutations)?;
    let dirty_region = ui_runtime
        .dirty_state()
        .render_region(ui_runtime.active_screen());
    let mode = render_mode_for_dirty_region(dirty_region);
    let report = render.renderer.flush(&mut render.framebuffer, mode)?;
    render.last_ui_renderer = report.renderer.to_string();
    match report.mode {
        RenderMode::FullFrame => display.flush_full_frame(&render.framebuffer)?,
        RenderMode::HudRegion => {
            display.flush_region(&render.framebuffer, router::status_bar_region())?
        }
        RenderMode::Region(region) => display.flush_region(&render.framebuffer, region)?,
    }
    let screen_changed = screen_changed_if_needed(&mut render.last_active_screen, ui_runtime);
    ui_runtime.mark_clean();
    render.frames += 1;
    Ok(screen_changed)
}

fn render_mode_for_dirty_region(region: Option<crate::render_contract::DirtyRegion>) -> RenderMode {
    match region {
        Some(region) if region == router::status_bar_region() => RenderMode::HudRegion,
        Some(region) => RenderMode::Region(region),
        None => RenderMode::FullFrame,
    }
}

fn active_scene_graph(ui_runtime: &UiRuntime, now_ms: u64) -> SceneGraph {
    let active = components::screens::scene_for_screen(
        ui_runtime.active_screen(),
        ui_runtime.snapshot(),
        ui_runtime.focus_index(),
        ui_runtime.selected_contact(),
    );
    let mut chrome = components::screens::chrome::chrome_for_screen(
        ui_runtime.active_screen(),
        ui_runtime.snapshot(),
        ui_runtime.focus_index(),
        ui_runtime.selected_contact(),
    );
    chrome.status.time = elapsed_time_label(now_ms);
    let modal_stack = active.modal.clone().into_iter().collect();
    SceneGraph {
        hud: HudScene {
            status: chrome.status,
            footer_text: chrome.footer_text,
        },
        active,
        history: ui_runtime
            .stack()
            .iter()
            .copied()
            .map(|route| crate::scene::ScenePushFrame {
                route,
                params: crate::scene::RouteParams::default(),
                cached_state: crate::scene::SceneCacheEntry::Discarded,
            })
            .collect(),
        modal_stack,
        global_clock: GlobalClock {
            started_ms: 0,
            now_ms,
        },
    }
}

fn elapsed_time_label(now_ms: u64) -> String {
    let total_seconds = now_ms / 1_000;
    let minutes = (total_seconds / 60).min(99);
    let seconds = total_seconds % 60;
    format!("{minutes:02}:{seconds:02}")
}

fn health_event(ui_runtime: &UiRuntime, render: &RenderState, button_events: usize) -> UiEvent {
    UiEvent::Health(UiHealth {
        frames: render.frames(),
        button_events,
        last_ui_renderer: render.last_ui_renderer().to_string(),
        active_screen: ui_runtime.active_screen(),
        full_snapshots: ui_runtime.full_snapshots,
        patches_per_domain: ui_runtime.patches_per_domain.clone(),
    })
}

fn screen_changed_if_needed(
    last_active_screen: &mut Option<UiScreen>,
    ui_runtime: &UiRuntime,
) -> Option<UiEvent> {
    let active_screen = ui_runtime.active_screen();
    if last_active_screen
        .map(|screen| screen != active_screen)
        .unwrap_or(true)
    {
        let chrome = components::screens::chrome::chrome_for_screen(
            active_screen,
            ui_runtime.snapshot(),
            ui_runtime.focus_index(),
            ui_runtime.selected_contact(),
        );
        let event = Some(UiEvent::ScreenChanged(UiScreenChanged {
            screen: active_screen,
            title: chrome.title,
        }));
        *last_active_screen = Some(active_screen);
        return event;
    }
    None
}
