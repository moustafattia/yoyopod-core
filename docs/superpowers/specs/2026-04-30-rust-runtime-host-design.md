# Rust Runtime Host Design

**Date:** 2026-04-30
**Owner:** Moustafa
**Status:** Draft for review
**Target hardware:** Raspberry Pi Zero 2W, dev lane

---

## 1. Problem

YoYoPod has already moved several runtime domains toward Rust. `yoyopod-voip-host`
owns Liblinphone runtime state, `yoyopod-media-host` owns the mpv/media backend
shape, and `yoyopod-ui-host` owns the Whisplay UI host path. The remaining app
process is still Python. It owns boot, loop cadence, worker supervision, event
routing, app snapshots, shutdown, and the cross-domain side effects between UI,
media, and calls.

The next migration goal is to flip the top-level runtime owner to Rust for the
Pi dev lane. The first milestone should boot the core device loop without a
long-running Python app runtime. Python CLI and deploy tooling may remain, but
the process running on the board should be Rust-owned.

---

## 2. Goals

- Add a new Rust runtime crate at `yoyopod_rs/runtime`.
- Provide a `yoyopod-runtime` binary that can act as the dev-lane service
  entrypoint.
- Start and supervise the existing Rust `ui`, `media`, and `voip` host workers.
- Own process startup, shutdown, signal handling, logging, status, event routing,
  loop cadence, and the composed app snapshot.
- Boot the Pi dev lane with Whisplay UI, one-button input, media playback, VoIP
  registration/call state, clean shutdown, and visible degraded states.
- Keep this first milestone narrow enough to validate on target hardware.

---

## 3. Non-goals

- Do not run `YoyoPodApp` or any other long-running Python app runtime process in
  this milestone.
- Do not require full Python runtime parity.
- Do not port cloud voice, cellular/GPS, advanced PiSugar/watchdog policy,
  screenshot signal handlers, or production slot packaging in this milestone.
- Do not rewrite `media-host`, `voip-host`, or `ui-host` as in-process libraries
  before the worker-supervised runtime is proven.
- Do not rename the final service entrypoint to `yoyopod` yet. Keep the first
  binary explicit as `yoyopod-runtime`.

---

## 4. Architecture Decision

Create a Rust top-level runtime host:

```text
yoyopod_rs/runtime -> yoyopod-runtime
  owns process lifecycle, config, event queue, state, loop, status, shutdown
  supervises existing Rust workers over NDJSON stdio

  -> yoyopod-ui-host
     owns Whisplay rendering, one-button input, UI navigation model

  -> yoyopod-media-host
     owns mpv/media backend, playback state, playlists, recents

  -> yoyopod-voip-host
     owns Liblinphone, registration, call/message/voice-note snapshots
```

This keeps the existing process contracts intact while moving the app runtime
authority out of Python. It is intentionally not a monolithic Rust binary yet.
The current worker boundaries are useful because they are already tested,
inspectable over stdio, and aligned with the domains that have moved to Rust.

The runtime crate should be named `runtime`, not `core`, because it owns the
long-running runtime process. A future `yoyopod_rs/core` crate can still exist
for shared Rust primitives if the codebase needs one later.

---

## 5. Component Split

The runtime crate should start with these focused modules:

- `main.rs`: CLI args, config path, hardware mode, signal handling, startup and
  shutdown markers.
- `config.rs`: minimal config loading for Whisplay, media, VoIP, brightness,
  startup volume, paths, and worker binary locations.
- `protocol.rs`: shared NDJSON envelope helpers compatible with the worker
  schema used by `media-host`, `voip-host`, and `ui-host`.
- `worker.rs`: process supervisor for `ui`, `media`, and `voip`, including
  child start, stdin/stdout handling, restart state, message draining, command
  send, and bounded shutdown.
- `event.rs`: typed internal events translated from worker envelopes.
- `state.rs`: composed runtime state and the app snapshot sent to `ui-host`.
- `loop.rs`: coordinator loop that drains worker messages, applies events, sends
  UI snapshots/ticks, and chooses sleep cadence.
- `status.rs`: status snapshot for dev-lane validation and logs.
- `logging.rs`: Rust logging setup that preserves deploy-readable startup and
  shutdown markers.

Each file should have one clear owner. The runtime should not become a generic
dump for all future domain logic. Domain truth remains in the domain workers
until a separate design approves a deeper merge.

---

## 6. Startup Flow

The first Rust dev-lane startup flow is:

1. `yoyopod-runtime` parses CLI options and resolves config paths.
2. Logging is configured and the canonical startup marker is emitted.
3. Minimal configuration is loaded.
4. The runtime starts workers in this order: `ui`, `media`, `voip`.
5. The runtime waits for `ui.ready`, `media.ready`, and `voip.ready`.
6. The runtime sends startup commands:
   - `media.configure`
   - `media.start`
   - `voip.configure`
   - `voip.register`
   - `ui.set_backlight`
   - initial `ui.runtime_snapshot`
7. The runtime enters the coordinator loop.

`ui-host` is mandatory for this milestone. If it fails to start or exits during
boot, `yoyopod-runtime` exits non-zero. `media-host` and `voip-host` may degrade
after bounded startup attempts so the UI can show the device is alive.

---

## 7. Steady-State Flow

The runtime uses one internal event queue as the replacement for Python's
`scheduler -> bus -> ui` ownership model.

Worker stdout NDJSON is decoded into typed `RuntimeEvent` values:

- `voip.snapshot` updates registration, call, message, and voice-note state.
- `media.snapshot` updates playback, library, playlist, and recent-track state.
- `ui.input` records user activity and updates wake/idle state.
- `ui.intent` becomes a runtime command such as play/pause, dial, hangup,
  voice-note action, or shutdown.
- `ui.screen_changed` updates the current screen name in runtime state.

After state-changing events, the runtime sends a fresh `ui.runtime_snapshot`.
Each loop tick also sends `ui.tick` so Whisplay input polling and rendering stay
active.

State ownership is explicit:

- VoIP truth stays in `voip-host`.
- Media truth stays in `media-host`.
- UI navigation/rendering truth stays in `ui-host`.
- The Rust runtime owns the composed app snapshot and cross-domain side effects,
  including pausing media during active calls when the configured policy enables
  it.

---

## 8. Error Handling And Recovery

Recovery is bounded in the first milestone.

- `ui-host` startup failure or runtime exit is fatal to `yoyopod-runtime`.
- `media-host` failure degrades media state and is exposed through status/UI.
- `voip-host` failure degrades communication state and is exposed through
  status/UI.
- `media-host` and `voip-host` may be restarted with capped backoff.
- Malformed worker envelopes are logged and counted. They do not crash the
  runtime unless they prevent mandatory `ui-host` boot.
- Worker commands have bounded send and shutdown waits.
- Shutdown sends `worker.stop` or domain-specific shutdown commands, waits
  briefly, then terminates remaining child processes.
- `SIGTERM` and `Ctrl+C` use the same graceful shutdown path.

The Python runtime's full recovery service, watchdog integration, screenshot
signals, and power safety events remain separate follow-up migrations.

---

## 9. Status And Logging

The runtime must emit startup and shutdown markers that remain easy for deploy
and status tooling to recognize. It should also expose a status snapshot with:

- process uptime
- current screen
- media worker state and playback state
- VoIP worker state and registration/call state
- UI worker state
- loop cadence and last loop duration
- worker restart counts
- protocol error counts
- last degraded reason per domain

For the first milestone this status can be log-readable and test-readable. A
later implementation can add a Unix socket, JSON status file, or CLI command if
the dev workflow needs it.

---

## 10. Testing Strategy

Rust unit tests should cover:

- envelope parsing and encoding
- unsupported schema handling
- worker supervisor state transitions
- event translation from worker envelopes
- state reduction into composed runtime snapshots
- command routing from UI intents to media/VoIP commands
- loop cadence decisions
- bounded shutdown decisions

Rust integration tests should use mock worker processes and prove:

- boot waits for ready events
- startup commands are sent in the expected order
- UI snapshots are emitted after state changes
- media degradation keeps the runtime alive
- VoIP degradation keeps the runtime alive
- UI failure exits the runtime
- malformed worker output is counted
- graceful shutdown stops children

Existing worker contract tests for `ui-host`, `media-host`, and `voip-host`
should remain green.

Local gates before commit and push remain:

```bash
uv run python scripts/quality.py gate
uv run pytest -q
cargo fmt --manifest-path yoyopod_rs/Cargo.toml --check
cargo clippy --manifest-path yoyopod_rs/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path yoyopod_rs/Cargo.toml
```

Pi dev-lane validation should confirm:

- the Rust runtime binary runs as the dev service entrypoint
- Whisplay UI boots
- one-button input is handled
- media worker starts and can play/pause
- VoIP worker registers or degrades visibly
- shutdown is clean and bounded

Rust binaries for target hardware should follow the existing artifact rule:
commit and push first, then use the GitHub Actions artifact for the exact commit
under test. Do not build Rust binaries on the Pi Zero 2W unless explicitly
overridden.

---

## 11. Follow-Up Migration Order

After this first milestone is working on the Pi dev lane, the likely follow-up
order is:

1. Harden runtime status and dev-lane service integration.
2. Port essential power telemetry and screen sleep behavior.
3. Port screenshots and freeze diagnostics.
4. Port cellular/GPS/network state.
5. Port cloud voice orchestration.
6. Decide whether worker crates should stay separate processes or merge into
   library crates for a monolithic Rust process.
7. Package the Rust runtime into the prod slot flow.

Each follow-up should be its own design or implementation plan if it changes
ownership boundaries.

---

## 12. Success Criteria

The first milestone is complete when:

- `yoyopod_rs/runtime` exists in the Rust workspace.
- `yoyopod-runtime` can boot on the Pi dev lane without a long-running Python app
  runtime.
- The runtime supervises `ui`, `media`, and `voip` workers.
- Whisplay UI and one-button input work through the Rust UI host.
- Media and VoIP worker snapshots update the composed app snapshot.
- Media and VoIP failures degrade visibly instead of crashing the runtime.
- UI failure is fatal and produces clear logs.
- Graceful shutdown stops all child workers within bounded waits.
- Local Python and Rust gates pass.
- Target validation proves the committed branch and exact artifact on the dev
  lane.

---

## 13. Final Recommendation

Proceed with a Rust top-level runtime host in `yoyopod_rs/runtime`. It should
supervise the existing Rust workers over their current NDJSON protocols and own
the Pi dev-lane app process. Keep the first milestone focused on the core device
loop: Whisplay UI, one-button input, media playback, VoIP registration/call
state, status, logging, and clean shutdown. Defer broader Python parity until
this ownership flip is proven on hardware.
