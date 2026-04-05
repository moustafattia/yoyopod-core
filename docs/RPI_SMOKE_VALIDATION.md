# Raspberry Pi Smoke Validation

This guide separates the validation work that is safe in CI from the checks that still require Raspberry Pi hardware, Mopidy, and a reachable SIP account.

## Validation Layers

### 1. CI-safe Python checks

Run these anywhere:

```bash
uv sync --extra dev
uv run pytest -q
```

This covers the pure-Python and simulation-mode regression suite.

### 2. Raspberry Pi core hardware smoke

Run this on the target Raspberry Pi after pulling the latest branch:

```bash
uv run python scripts/pi_smoke.py
uv run python scripts/pi_smoke.py --with-power --with-rtc
uv run python scripts/pi_smoke.py --with-lvgl-soak
```

What it checks:

- target environment information
- display initialization on real hardware
- matching input adapter construction and start/stop
- optional LVGL transition and sleep/wake soak when requested

Expected result:

- `display` should report a real hardware adapter, not simulation
- `input` should report the active interaction profile plus semantic capabilities for the attached hardware

### 3. Raspberry Pi service smoke

Add Mopidy and SIP checks when the services are expected to be available:

```bash
uv run python scripts/pi_smoke.py --with-mopidy --with-voip
uv run python scripts/pi_smoke.py --with-power --with-rtc --with-mopidy --with-voip
```

What it checks:

- PiSugar battery telemetry and RTC state when requested
- Mopidy JSON-RPC connectivity using `config/yoyopod_config.yaml`
- `linphonec` startup and SIP registration using `config/voip_config.yaml`

Useful flags:

- `--mopidy-timeout 10`
- `--voip-timeout 15`
- `--verbose`

## Manual Follow-up Checks

### Full application startup

```bash
uv run python yoyopod.py
```

Verify:

- the home/menu UI renders on the target display
- button input navigates screens correctly
- the app returns cleanly on `Ctrl+C`

### VoIP registration drill

```bash
uv run python scripts/check_voip_registration.py
```

Use this when you want a registration-only pass with detailed logs.

### Incoming call debug drill

```bash
uv run python scripts/debug_incoming_call.py
```

Use this when SIP registration works but incoming-call parsing or callback delivery looks wrong.

### Whisplay display-only debug

```bash
uv run python test_hal_whisplay.py
```

Use this only on a Pi with the Whisplay hardware attached. It is a manual hardware smoke script, not part of CI.

### Whisplay gesture tuning

```bash
uv run python scripts/whisplay_tune.py
uv run python scripts/whisplay_tune.py --double-tap-ms 240 --long-hold-ms 900
```

Use this when button feel needs tuning on the real device. It listens for the semantic Whisplay gestures, prints every detected `advance` / `select` / `back` event with timing detail, and can apply temporary timing overrides without editing `config/yoyopod_config.yaml`.

Useful flags:

- `--duration-seconds 45`
- `--debounce-ms 75`
- `--double-tap-ms 240`
- `--long-hold-ms 900`
- `--verbose`

### LVGL Whisplay soak

```bash
uv run python scripts/lvgl_soak.py
uv run python scripts/lvgl_soak.py --cycles 3 --hold-seconds 0.3
```

Use this when Whisplay rendering feels fast but you still want a hardware pass for:

- repeated routed screen transitions
- sleep/wake recovery
- LVGL-only corruption or stuck redraw issues

### PiSugar RTC drill

```bash
uv run python scripts/pisugar_rtc.py status
uv run python scripts/pisugar_rtc.py sync-to-rtc
```

Use this when you want a focused RTC read/sync pass without running the full app.

### PiSugar power drill

```bash
uv run python scripts/pisugar_power.py
```

Use this when you want a focused battery, charging, RTC, shutdown-threshold, and watchdog readout without the full smoke flow.

## Suggested Order On Hardware

1. `uv sync --extra dev`
2. `uv run pytest -q`
3. `uv run python scripts/pi_smoke.py`
4. `uv run python scripts/pi_smoke.py --with-mopidy --with-voip`
5. `uv run python scripts/pi_smoke.py --with-lvgl-soak`
6. `uv run python yoyopod.py`

## Failure Triage

- `display` fails: check attached HAT, driver/library install, and `display.hardware` config
- `input` fails: check the matching display adapter initialized correctly first
- `mopidy` fails: verify Mopidy is running and reachable at the configured host/port
- `voip` fails: verify `linphonec`, SIP credentials, network reachability, and audio device configuration

## Notes

- The smoke script exits non-zero if any requested check fails.
- CI intentionally does not run hardware-in-the-loop checks.
- The hardware smoke script is meant to be quick. Use the manual drills above when you need deeper debugging.
