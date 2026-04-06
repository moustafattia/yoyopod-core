# RPi Deploy Plugin — Design Spec

## Problem

The inner loop for developing YoyoPod on Raspberry Pi is too slow. Every code change requires manually running SSH commands to push, pull, kill processes, restart the app, and tail logs. There is no structured way for Claude Code to interact with the Pi during development.

## Solution

A Claude Code plugin called `rpi-deploy` providing five slash-command skills that automate deployment, process management, and log retrieval over SSH.

## Plugin Structure

```
rpi-deploy/
├── plugin.json
├── skills/
│   ├── deploy.md          # /deploy — git-based deploy
│   ├── sync.md            # /sync — rsync dirty tree, restart
│   ├── logs.md            # /logs — tail app output
│   ├── restart.md         # /restart — kill + relaunch
│   └── pi-status.md       # /pi-status — health check
└── config/
    └── pi-target.example.yaml
```

All skills are prompt-only (no custom code). Each skill instructs Claude to orchestrate Bash tool calls over SSH.

## Per-Project Config

Each project that uses this plugin places a `pi-deploy.yaml` in its root:

```yaml
# Connection
host: rpi-zero                          # SSH host (from ~/.ssh/config)
user: pi                                # SSH user (optional)

# Project paths (on the Pi)
remote_dir: /home/pi/yoyo-py
venv: /home/pi/yoyo-py/.venv

# App lifecycle
start_cmd: python yoyopod.py
kill_processes:
  - python
  - linphonec

# Logging (must match app's loguru config)
log_file: /home/pi/yoyo-py/logs/yoyopod.log
error_log_file: /home/pi/yoyo-py/logs/yoyopod_errors.log
pid_file: /tmp/yoyopod.pid
startup_marker: "YoyoPod starting"

# Sync
rsync_exclude:
  - .git/
  - __pycache__/
  - "*.pyc"
  - .venv/
  - logs/
```

Skills read this file at invocation time via the Read tool. The `host` field relies on `~/.ssh/config` for key auth, keeping secrets out of the repo.

## Skill Specifications

### /deploy — Git-based deploy

**Trigger phrases:** "deploy to pi", "push and deploy", `/deploy`

**Behavior:**

1. Read `pi-deploy.yaml` for connection and path config.
2. Check local git status. If there are uncommitted changes, warn the user and suggest `/sync` instead.
3. Run `git push` from the local working directory.
4. SSH: `cd $remote_dir && git pull origin main`.
5. SSH: kill processes listed in `kill_processes` via `killall -9`.
6. SSH: activate venv and run `start_cmd` in background via `nohup ... > /dev/null 2>&1 &`.
7. Wait 3 seconds, then verify:
   - Check PID file exists and process is alive (`kill -0 $(cat $pid_file)`).
   - Grep the log file for `startup_marker` matching the new PID.
8. Report success with commit hash, PID, and startup confirmation — or report failure with the last 20 lines of the log.

### /sync — Quick rsync deploy (no commit)

**Trigger phrases:** "sync to pi", "quick deploy", `/sync`

**Behavior:**

1. Read `pi-deploy.yaml`.
2. Build rsync exclude flags from `rsync_exclude` list.
3. Run `rsync -avz --delete` from local working tree to `$user@$host:$remote_dir`, with excludes.
4. Report files transferred/changed.
5. Kill processes (same as `/deploy` step 5).
6. Start app and verify (same as `/deploy` steps 6–8).

### /logs — Tail app output

**Trigger phrases:** "show pi logs", "what's happening on the pi", `/logs`

**Behavior:**

1. Read `pi-deploy.yaml` for `log_file`, `error_log_file`, and `host`.
2. Determine which log file to read:
   - Default: `log_file` (main log).
   - If `--errors` flag: use `error_log_file`.
3. Determine line count: default 50, override with argument (e.g., `/logs 100`).
4. SSH: `tail -n $count $log_file`.
5. If `--filter` argument provided (e.g., `/logs --filter voip`), SSH: `grep -i "$filter" $log_file | tail -n $count`.
6. Return log output into the conversation. Claude can then be asked follow-up questions about the log content (e.g., "why did the call drop?").

**Subsystem filter shorthand:** Since YoyoPod logs include subsystem tags (`voip`, `music`, `coord`, `ui`, `power`, `config`, `app`, `core`), the filter can match these directly: `/logs --filter voip` greps for lines containing `voip`.

### /restart — Kill and relaunch

**Trigger phrases:** "restart the app", "reboot yoyopod", `/restart`

**Behavior:**

1. Read `pi-deploy.yaml`.
2. SSH: check PID file — if it exists and process is alive, kill that specific PID first.
3. SSH: `killall -9` for each process in `kill_processes` (catches orphans).
4. SSH: activate venv and run `start_cmd` in background.
5. Verify startup (same as `/deploy` steps 7–8).

### /pi-status — Health check

**Trigger phrases:** "is the pi running?", "check pi status", `/pi-status`

**Behavior:**

1. Read `pi-deploy.yaml`.
2. SSH connectivity check with 5-second timeout.
3. SSH: check PID file and whether the process is alive.
4. SSH: `uptime` (load average) and `free -m` (memory — important on 416MB Pi Zero).
5. SSH: check if `linphonec` is running (`pgrep linphonec`).
6. SSH: last 5 lines of `log_file` for recent activity.
7. Present a compact dashboard:

```
Pi Status: rpi-zero
  SSH:        connected
  App:        running (pid 1234, uptime 2h 15m)
  linphonec:  running (pid 1235)
  Memory:     198/416 MB used
  Last log:   2026-04-06 14:23:45 | INFO | voip | Registration successful
```

## Logging Contract

The skills depend on YoyoPod's loguru-based logging implementation providing:

| Feature | Implementation | Used by |
|---|---|---|
| Main log file | `logs/yoyopod.log`, 5MB rotation, 3-day retention, gzip | `/logs`, `/deploy`, `/sync` |
| Error log file | `logs/yoyopod_errors.log`, ERROR+, 2MB rotation, 7-day retention | `/logs --errors` |
| PID file | `/tmp/yoyopod.pid`, atexit cleanup | `/deploy`, `/sync`, `/restart`, `/pi-status` |
| Startup marker | `"===== YoyoPod starting (version=X, pid=Y) ====="` | `/deploy`, `/sync`, `/restart` |
| Shutdown marker | `"===== YoyoPod shutting down (pid=Y) ====="` | `/restart` |
| Subsystem tags | 6-char tags: voip, music, coord, ui, power, config, app, core | `/logs --filter` |
| Synchronous writes | `enqueue=False` | `/logs` (near-realtime) |
| Full tracebacks | `backtrace=True`, `diagnose=True` | `/logs` (debugging) |
| Exception hooks | Main thread + worker thread unhandled exceptions logged | `/logs` (crash diagnosis) |

### /screenshot — Capture display output

**Trigger phrases:** "take a screenshot", "show me the screen", `/screenshot`

**Default capture mode:**

- **LVGL readback first:** Captures directly from LVGL's internal object tree via `lv_snapshot_take()`. This shows what LVGL *actually rendered*.
- **Fallback:** If readback is unavailable, the app falls back to the adapter's screenshot method.

**Behavior:**

1. Read `pi-deploy.yaml` for `host`, `user`, `pid_file`, and `screenshot_path`.
2. Send `SIGUSR1` to the app process.
3. Wait 1 second for the screenshot to be written.
4. SCP the screenshot PNG from the Pi to a local temp file.
5. Read the local PNG file using the Read tool (Claude is multimodal — it can see images).
6. Present the screenshot in the conversation. Claude can then answer questions about the display state.

**App-side requirements (in YoyoPod):**

- `SIGUSR1` handler: uses LVGL readback first and falls back to the adapter screenshot method.
- `SIGUSR2` handler: remains available as a legacy shadow-first debug path.
- `lv_conf.h` must enable `LV_USE_SNAPSHOT 1`.
- C shim must expose a `yoyopy_lvgl_snapshot()` function.

**Config addition to `pi-deploy.yaml`:**

```yaml
screenshot_path: /tmp/yoyopod_screenshot.png
```

## Future Considerations

- **UART transport:** Add `transport: uart` and `serial_port` fields to config. Skills would use `picocom` or `screen` instead of SSH. Not in v1.
- **Generalization:** Currently YoyoPod-specific. To support other Pi projects, the config file already captures all project-specific details. The skills are project-agnostic — they only reference config values.
- **File watching / auto-sync:** A hook that auto-runs `/sync` on file save. Deferred — adds complexity and may be annoying.
- **Log streaming:** Instead of snapshot tailing, a persistent SSH session streaming logs in realtime. Deferred — current tail approach is simpler and sufficient.

## Out of Scope

- JSON-structured logs (Claude reads formatted text fine)
- Remote log shipping (syslog, etc.)
- Runtime log level switching
- MCP server (skills over Bash/SSH are simpler)
- Hooks or auto-deploy triggers
