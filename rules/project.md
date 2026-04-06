# Project Overview

YoyoPod is an iPod-inspired Raspberry Pi application combining SIP calling (VoIP via linphonec) and Mopidy-based music playback behind a small-screen, button-driven UI. Target hardware is Raspberry Pi Zero 2W (416 MB RAM).

Three display/input modes: Pimoroni DisplayHATMini, PiSugar Whisplay, and browser-based simulation.

## Common Commands

```bash
# Run the app
python yoyopod.py              # Production (requires Pi hardware)
python yoyopod.py --simulate   # Simulation mode (browser UI at localhost:5000)

# Tests
pytest
pytest tests/test_fsm_runtime.py
pytest -v

# Code quality
black .
ruff check .
mypy yoyopy/

# Install dev dependencies
pip install -e ".[dev]"
```

## Configuration

All config in `config/` directory (tracked in repo):
- `yoyopod_config.yaml` -- display hardware, Mopidy host/port, auto-resume
- `voip_config.yaml` -- SIP account, transport, STUN, HA1 hash auth
- `contacts.yaml` -- contact list and speed dial

## Current Gaps

- Settings UI is still not implemented
- Hardware-required validation still needs real Pi coverage beyond CI and simulation
