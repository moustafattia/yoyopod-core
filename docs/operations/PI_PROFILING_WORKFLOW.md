# Raspberry Pi Profiling Workflow

**Status: in transition.**

The Python profiling helpers (`yoyopod dev profile …`,
`yoyopod build voice-worker`, `yoyopod remote validate`,
`yoyopod remote sync`) were deleted in Round 0 of the CLI rebuild.
Hardware profiling now uses native Rust tools directly. See
[`CLI_REBUILD_ROUNDS.md`](CLI_REBUILD_ROUNDS.md).

## When to profile

Use this workflow when you want to answer:

- did the branch make startup slower?
- is the runtime spending more time in one worker?
- is the Pi staying responsive during music, VoIP, and navigation soaks?
- is memory growth coming from a specific worker or a native dependency?

The Pi Zero 2 W is the source of truth. Desktop simulation is useful as
a fast triage, not as proof that the Pi will behave the same way.

## Deploy + capture loop

```bash
yoyopod target mode activate dev
yoyopod target deploy --branch <branch>           # or --sha <commit>
yoyopod target status
yoyopod target logs --follow                      # leave running during soak
```

After the run, snapshot logs:

```bash
yoyopod target logs --lines 500
yoyopod target status
```

## Responsiveness watchdog env vars

The runtime exports responsiveness diagnostics when these env vars are
set in the dev service environment (e.g. via `/etc/default/yoyopod-dev`):

```
YOYOPOD_RESPONSIVENESS_WATCHDOG_ENABLED=true
YOYOPOD_RESPONSIVENESS_STALL_THRESHOLD_SECONDS=5
YOYOPOD_RESPONSIVENESS_RECENT_INPUT_WINDOW_SECONDS=3
```

Setting them in the local shell before `yoyopod target deploy` does NOT
propagate to the systemd-managed runtime. Either edit the service's
`EnvironmentFile=` target on the Pi, or fold them into
`config/app/core.yaml`:

```yaml
diagnostics:
  responsiveness_watchdog_enabled: true
  responsiveness_stall_threshold_seconds: 5.0
```

That captures evidence bundles under `logs/responsiveness/` when the
loop stops advancing.

## Pre-worker baseline

Use this workflow before any architectural change that moves work
between workers. The goal is to capture responsiveness and PSS/RSS with
the current single-supervisor runtime so changes can be compared
against a real target-hardware baseline.

After deploy, run a one-hour mixed soak covering:

- idle screen wake/sleep
- music navigation and playback
- incoming or simulated VoIP state changes
- voice command path with current local settings
- cellular/network reconnect or status polling
- low-risk power/status screen navigation

Record these fields from log output and any responsiveness evidence
bundles. Keep null or missing values too:

- `responsiveness_input_to_action_p95_ms`
- `responsiveness_action_to_visible_p95_ms`
- `runtime_loop_gap_seconds`
- `runtime_main_thread_drain_seconds`
- `runtime_blocking_span_name`
- `runtime_blocking_span_seconds`
- `process_memory_rss_kb`
- `process_memory_pss_kb`
- `workers`

The baseline is not pass/fail. Use it to identify the top stalls and
memory pressure, then keep the raw notes with the branch or release
artifacts they describe.

## Cloud voice measurement

When measuring cloud voice changes:

1. Deploy the change as usual via `yoyopod target deploy`.
2. Capture status and process memory for each scenario:
   - voice disabled
   - cloud voice configured and one transcription attempted
   - voice worker idle with mock provider
   - voice worker cloud STT request
   - voice worker cloud TTS request
   - provider unavailable or missing credentials
   - worker crash and supervisor restart
3. Fields to record:
   - supervisor PSS/RSS
   - voice worker PSS/RSS
   - total process tree PSS/RSS
   - `responsiveness_input_to_action_p95_ms`
   - `responsiveness_action_to_visible_p95_ms`
   - `runtime_main_thread_drain_seconds`
   - voice worker pending requests
   - voice worker restart count
   - protocol errors and dropped messages

Cloud voice mode is acceptable only if STT/TTS requests avoid UI-loop
stalls and total PSS stays within the Pi Zero 2W service budget.

## Native profiling tools

For Rust runtime / worker profiling on the Pi:

- `perf` (system profile) — `perf record -g -p $(pgrep yoyopod-runtime)`
- `cargo flamegraph` — install via `cargo install flamegraph` on the
  dev machine; produce flamegraphs from `perf` data captured on the Pi
- `samply` — modern flamegraph-friendly profiler
- `heaptrack` / `valgrind --tool=massif` — for memory growth questions
- `journalctl -u yoyopod-dev.service -o json` — structured runtime
  events
- `top -p $(pgrep yoyopod-runtime)` / `pidstat` — quick CPU and memory
  baseline

Capture data on the Pi, copy it to the dev machine, and post-process
locally.

### perf examples

For a live runtime process:

```bash
ssh <user>@<host>
perf stat -p "$(pgrep yoyopod-runtime)" -I 1000
```

For a bounded run (locally on the dev machine for triage):

```bash
mkdir -p logs/profiles
perf record -F 99 -g -o logs/profiles/perf-runtime.data \
  device/target/release/yoyopod-runtime --config-dir config
perf report -g -i logs/profiles/perf-runtime.data
```

## Practical sequence

When checking whether a change regressed the Pi:

1. Run focused Rust build checks for the crates you changed
   (`cargo check -p <crate>`).
2. `yoyopod target deploy --branch <branch>` (always uses the CI
   artifact).
3. Watch `yoyopod target logs --follow --filter coord` and
   `--filter comm` warnings first.
4. If the Pi looks CPU-bound, attach `perf` or `samply` to the live
   runtime PID.
5. If memory looks suspect, use `pidstat -r` and capture PSS/RSS at
   intervals.
6. Report branch, exact commit SHA, CI artifact name, target hardware,
   duration, tool used, and the raw data location.

That order keeps the cheapest and most repo-specific signals first, and
only drops to heavier system profilers when the runtime's own
diagnostics are not enough.
