---
name: yoyopod-status
description: Health check for Raspberry Pi - connectivity, processes, memory, recent logs
disable-model-invocation: true
allowed-tools:
  - Read
  - Bash(uv run python scripts/pi_remote.py:*)
---

## Config

Use `deploy/pi-deploy.yaml` as the shared deploy contract and `deploy/pi-deploy.local.yaml` for machine-specific overrides such as host, SSH user, project dir, and branch. `scripts/pi_remote.py` merges them directly, and `uv run python scripts/pi_remote.py config edit` is the preferred way to create or update the local override.

If the file does not exist yet, run `uv run python scripts/pi_remote.py config edit` first. That command creates `deploy/pi-deploy.local.yaml` automatically before opening it.

## Steps

1. **Run the helper command.**
   ```bash
   uv run python scripts/pi_remote.py status
   ```

2. **Present the result.** Prefer a compact summary with:
   - git branch and commit
   - music backend status
   - YoyoPod service status
   - PiSugar server status
   - PID file state
   - latest startup marker
   - top memory processes

3. **If the app is not running,** explicitly suggest:
   ```text
   Run /yoyopod-restart to start the app.
   ```
