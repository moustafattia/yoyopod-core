# RPi Deploy Plugin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Claude Code plugin with five slash commands (`/deploy`, `/sync`, `/logs`, `/restart`, `/pi-status`) that automate the Raspberry Pi development inner loop over SSH.

**Architecture:** A standalone Claude Code plugin directory (`rpi-deploy/`) containing a `plugin.json` manifest, five command markdown files, an example config, and a project-specific `pi-deploy.yaml` in the yoyo-py repo. Commands are prompt-only — each instructs Claude to orchestrate Bash tool calls (SSH, rsync, git) against the Pi. No custom code, no MCP server.

**Tech Stack:** Claude Code plugin system (plugin.json + command .md files), SSH, rsync, git, YAML config

---

## File Structure

```
C:\Workspace\my-ca\rpi-deploy/           # Plugin root (standalone repo)
├── .claude-plugin/
│   └── plugin.json                      # Plugin manifest
├── commands/
│   ├── deploy.md                        # /deploy command
│   ├── sync.md                          # /sync command
│   ├── logs.md                          # /logs command
│   ├── restart.md                       # /restart command
│   └── pi-status.md                     # /pi-status command
└── config/
    └── pi-deploy.example.yaml           # Example config for reference

C:\Workspace\my-ca\yoyo-py/
└── pi-deploy.yaml                       # YoyoPod-specific config (new file)
```

**Why commands/ not skills/:** These are user-invoked slash commands (the user types `/deploy`), not model-invoked skills (where Claude decides to use them). The commit-commands plugin uses this same pattern. Each command has `allowed-tools` frontmatter restricting what Bash commands it can run.

---

### Task 1: Plugin scaffold and manifest

**Files:**
- Create: `C:\Workspace\my-ca\rpi-deploy\.claude-plugin\plugin.json`

- [ ] **Step 1: Create the plugin directory structure**

```bash
mkdir -p "C:\Workspace\my-ca\rpi-deploy\.claude-plugin"
mkdir -p "C:\Workspace\my-ca\rpi-deploy\commands"
mkdir -p "C:\Workspace\my-ca\rpi-deploy\config"
```

- [ ] **Step 2: Write plugin.json**

Write to `C:\Workspace\my-ca\rpi-deploy\.claude-plugin\plugin.json`:

```json
{
  "name": "rpi-deploy",
  "description": "Deploy, debug, and manage applications running on Raspberry Pi over SSH. Provides /deploy, /sync, /logs, /restart, and /pi-status commands.",
  "author": {
    "name": "Moustafa Attia"
  }
}
```

- [ ] **Step 3: Initialize git repo**

```bash
cd "C:\Workspace\my-ca\rpi-deploy"
git init
```

- [ ] **Step 4: Commit scaffold**

```bash
cd "C:\Workspace\my-ca\rpi-deploy"
git add .claude-plugin/plugin.json
git commit -m "feat: scaffold rpi-deploy plugin with manifest"
```

---

### Task 2: Example config and project config

**Files:**
- Create: `C:\Workspace\my-ca\rpi-deploy\config\pi-deploy.example.yaml`
- Create: `C:\Workspace\my-ca\yoyo-py\pi-deploy.yaml`

- [ ] **Step 1: Write the example config**

Write to `C:\Workspace\my-ca\rpi-deploy\config\pi-deploy.example.yaml`:

```yaml
# pi-deploy.yaml — Place this file in your project root.
# Each field is read by the /deploy, /sync, /logs, /restart, and /pi-status commands.

# Connection (host should match an entry in ~/.ssh/config for key-based auth)
host: rpi-zero
user: pi                                  # optional, defaults to current user

# Project paths on the Pi
remote_dir: /home/pi/my-project
venv: /home/pi/my-project/.venv

# App lifecycle
start_cmd: python app.py
kill_processes:
  - python

# Logging (must match your app's log output paths)
log_file: /home/pi/my-project/logs/app.log
error_log_file: /home/pi/my-project/logs/app_errors.log
pid_file: /tmp/my-project.pid
startup_marker: "App starting"

# Sync exclusions for /sync (rsync --exclude patterns)
rsync_exclude:
  - .git/
  - __pycache__/
  - "*.pyc"
  - .venv/
  - logs/
```

- [ ] **Step 2: Write the YoyoPod project config**

Write to `C:\Workspace\my-ca\yoyo-py\pi-deploy.yaml`:

```yaml
# RPi Deploy config for YoyoPod
# Used by the rpi-deploy plugin commands: /deploy, /sync, /logs, /restart, /pi-status

host: rpi-zero
user: pi

remote_dir: /home/pi/yoyo-py
venv: /home/pi/yoyo-py/.venv

start_cmd: python yoyopod.py
kill_processes:
  - python
  - linphonec

log_file: /home/pi/yoyo-py/logs/yoyopod.log
error_log_file: /home/pi/yoyo-py/logs/yoyopod_errors.log
pid_file: /tmp/yoyopod.pid
startup_marker: "YoyoPod starting"

rsync_exclude:
  - .git/
  - __pycache__/
  - "*.pyc"
  - .venv/
  - logs/
  - node_modules/
  - "*.egg-info/"
```

- [ ] **Step 3: Commit the configs**

In the rpi-deploy repo:

```bash
cd "C:\Workspace\my-ca\rpi-deploy"
git add config/pi-deploy.example.yaml
git commit -m "feat: add example pi-deploy.yaml config"
```

In the yoyo-py repo:

```bash
cd "C:\Workspace\my-ca\yoyo-py"
git add pi-deploy.yaml
git commit -m "feat: add pi-deploy.yaml for rpi-deploy plugin"
```

---

### Task 3: /deploy command

**Files:**
- Create: `C:\Workspace\my-ca\rpi-deploy\commands\deploy.md`

- [ ] **Step 1: Write the deploy command**

Write to `C:\Workspace\my-ca\rpi-deploy\commands\deploy.md`:

````markdown
---
description: Git-based deploy to Raspberry Pi (push, pull, restart)
allowed-tools: Read, Bash(git status:*), Bash(git push:*), Bash(ssh:*)
---

## Config

Read the `pi-deploy.yaml` file in the current project root to get all connection and path settings. If the file does not exist, stop and tell the user to create one (see the rpi-deploy plugin's `config/pi-deploy.example.yaml` for the format).

Extract these values from the YAML:
- `host` — SSH host
- `user` — SSH user (optional, omit from SSH commands if not set)
- `remote_dir` — project directory on the Pi
- `venv` — virtualenv path on the Pi
- `start_cmd` — command to start the app
- `kill_processes` — list of process names to kill before restart
- `pid_file` — path to PID file on the Pi
- `log_file` — path to main log file on the Pi
- `startup_marker` — string to grep for in logs to confirm startup

Construct the SSH target as: `user@host` if user is set, otherwise just `host`.

## Steps

1. **Check local git status.** Run `git status`. If there are uncommitted or unstaged changes, warn the user: "You have uncommitted changes. Use `/sync` for a quick deploy without committing, or commit first." Stop and wait for the user's response.

2. **Push to remote.** Run `git push`.

3. **Pull on the Pi.** Run:
   ```
   ssh <target> "cd <remote_dir> && git pull origin main"
   ```

4. **Kill running processes.** For each process name in `kill_processes`, run:
   ```
   ssh <target> "killall -9 <process_name> 2>/dev/null; true"
   ```

5. **Start the app.** Run:
   ```
   ssh <target> "cd <remote_dir> && source <venv>/bin/activate && nohup <start_cmd> > /dev/null 2>&1 &"
   ```

6. **Verify startup.** Wait 3 seconds, then run these two checks:

   a. Check PID file exists and process is alive:
   ```
   ssh <target> "test -f <pid_file> && kill -0 \$(cat <pid_file>) 2>/dev/null && echo ALIVE || echo DEAD"
   ```

   b. Get the PID and check for startup marker in the log:
   ```
   ssh <target> "cat <pid_file> 2>/dev/null"
   ssh <target> "grep '<startup_marker>' <log_file> | tail -1"
   ```

7. **Report results.**
   - If ALIVE and startup marker found: report success with the PID and the startup log line.
   - If DEAD or no startup marker: report failure and show the last 20 lines of the log file:
     ```
     ssh <target> "tail -20 <log_file>"
     ```

Present results in a compact format:
```
Deploy complete:
  Commit:  <short hash from git push output>
  PID:     <pid>
  Status:  running
  Startup: <startup marker log line>
```

Or on failure:
```
Deploy FAILED:
  PID:    <pid or "not found">
  Status: not running
  Last 20 log lines:
  <log output>
```
````

- [ ] **Step 2: Test the command loads**

Install the plugin locally and verify it appears:

```bash
claude plugin add "C:\Workspace\my-ca\rpi-deploy"
```

Then in a Claude Code session, type `/deploy` and verify it shows up in the command list.

- [ ] **Step 3: Commit**

```bash
cd "C:\Workspace\my-ca\rpi-deploy"
git add commands/deploy.md
git commit -m "feat: add /deploy command — git-based deploy to Pi"
```

---

### Task 4: /sync command

**Files:**
- Create: `C:\Workspace\my-ca\rpi-deploy\commands\sync.md`

- [ ] **Step 1: Write the sync command**

Write to `C:\Workspace\my-ca\rpi-deploy\commands\sync.md`:

````markdown
---
description: Quick rsync deploy to Raspberry Pi (no commit needed)
allowed-tools: Read, Bash(rsync:*), Bash(ssh:*)
---

## Config

Read the `pi-deploy.yaml` file in the current project root to get all connection and path settings. If the file does not exist, stop and tell the user to create one (see the rpi-deploy plugin's `config/pi-deploy.example.yaml` for the format).

Extract these values from the YAML:
- `host` — SSH host
- `user` — SSH user (optional, omit from SSH commands if not set)
- `remote_dir` — project directory on the Pi
- `venv` — virtualenv path on the Pi
- `start_cmd` — command to start the app
- `kill_processes` — list of process names to kill before restart
- `pid_file` — path to PID file on the Pi
- `log_file` — path to main log file on the Pi
- `startup_marker` — string to grep for in logs to confirm startup
- `rsync_exclude` — list of patterns to exclude from sync

Construct the SSH target as: `user@host` if user is set, otherwise just `host`.

## Steps

1. **Rsync working tree to Pi.** Build the rsync command with all exclude patterns from `rsync_exclude`. For each pattern, add `--exclude '<pattern>'` to the command. Run:
   ```
   rsync -avz --delete --exclude '<pattern1>' --exclude '<pattern2>' ... ./ <target>:<remote_dir>/
   ```
   Note the trailing slashes — this syncs the contents of the current directory into `remote_dir`.

2. **Report sync results.** Show the rsync summary (files transferred, total size).

3. **Kill running processes.** For each process name in `kill_processes`, run:
   ```
   ssh <target> "killall -9 <process_name> 2>/dev/null; true"
   ```

4. **Start the app.** Run:
   ```
   ssh <target> "cd <remote_dir> && source <venv>/bin/activate && nohup <start_cmd> > /dev/null 2>&1 &"
   ```

5. **Verify startup.** Wait 3 seconds, then run these two checks:

   a. Check PID file exists and process is alive:
   ```
   ssh <target> "test -f <pid_file> && kill -0 \$(cat <pid_file>) 2>/dev/null && echo ALIVE || echo DEAD"
   ```

   b. Get the PID and check for startup marker in the log:
   ```
   ssh <target> "cat <pid_file> 2>/dev/null"
   ssh <target> "grep '<startup_marker>' <log_file> | tail -1"
   ```

6. **Report results.**
   - If ALIVE and startup marker found: report success with PID and startup log line.
   - If DEAD or no startup marker: report failure and show last 20 lines of the log:
     ```
     ssh <target> "tail -20 <log_file>"
     ```

Present results in a compact format:
```
Sync + restart complete:
  Files synced: <count from rsync>
  PID:          <pid>
  Status:       running
  Startup:      <startup marker log line>
```
````

- [ ] **Step 2: Commit**

```bash
cd "C:\Workspace\my-ca\rpi-deploy"
git add commands/sync.md
git commit -m "feat: add /sync command — rsync deploy to Pi"
```

---

### Task 5: /logs command

**Files:**
- Create: `C:\Workspace\my-ca\rpi-deploy\commands\logs.md`

- [ ] **Step 1: Write the logs command**

Write to `C:\Workspace\my-ca\rpi-deploy\commands\logs.md`:

````markdown
---
description: Tail application logs from Raspberry Pi
allowed-tools: Read, Bash(ssh:*)
argument-hint: "[line_count] [--errors] [--filter <subsystem>]"
---

## Config

Read the `pi-deploy.yaml` file in the current project root to get all connection and path settings. If the file does not exist, stop and tell the user to create one.

Extract these values from the YAML:
- `host` — SSH host
- `user` — SSH user (optional, omit from SSH commands if not set)
- `log_file` — path to main log file on the Pi
- `error_log_file` — path to error-only log file on the Pi

Construct the SSH target as: `user@host` if user is set, otherwise just `host`.

## Argument Parsing

Parse the arguments string provided after `/logs`:

- **Line count:** If a bare number is present (e.g., `/logs 100`), use it as the line count. Default: 50.
- **--errors flag:** If `--errors` is present, use `error_log_file` instead of `log_file`.
- **--filter value:** If `--filter <value>` is present, filter log lines by that value. Common values are subsystem tags: `voip`, `music`, `coord`, `ui`, `power`, `config`, `app`, `core`. But any string works.

Multiple flags can be combined: `/logs 100 --errors --filter voip`

## Steps

1. **Determine the log file.** Use `error_log_file` if `--errors` was specified, otherwise use `log_file`.

2. **Fetch log lines.**

   If `--filter` was specified, grep then tail:
   ```
   ssh <target> "grep -i '<filter>' <chosen_log_file> | tail -n <count>"
   ```

   If no filter, just tail:
   ```
   ssh <target> "tail -n <count> <chosen_log_file>"
   ```

3. **Handle empty results.** If the SSH command returns empty output:
   - Check if the log file exists: `ssh <target> "test -f <chosen_log_file> && echo EXISTS || echo MISSING"`
   - If MISSING: tell the user the log file doesn't exist — the app may not have been started yet.
   - If EXISTS but empty output: tell the user no matching lines were found. If a filter was used, suggest trying without the filter.

4. **Present the log output.** Return the raw log lines directly into the conversation. Do not summarize or truncate — the user (or Claude in a follow-up) needs the full lines for diagnosis.

After presenting the logs, remind the user they can ask follow-up questions about the log content, such as "why did the call drop?" or "what errors happened in the last minute?"
````

- [ ] **Step 2: Commit**

```bash
cd "C:\Workspace\my-ca\rpi-deploy"
git add commands/logs.md
git commit -m "feat: add /logs command — tail Pi app logs with filtering"
```

---

### Task 6: /restart command

**Files:**
- Create: `C:\Workspace\my-ca\rpi-deploy\commands\restart.md`

- [ ] **Step 1: Write the restart command**

Write to `C:\Workspace\my-ca\rpi-deploy\commands\restart.md`:

````markdown
---
description: Kill and relaunch the app on Raspberry Pi
allowed-tools: Read, Bash(ssh:*)
---

## Config

Read the `pi-deploy.yaml` file in the current project root to get all connection and path settings. If the file does not exist, stop and tell the user to create one.

Extract these values from the YAML:
- `host` — SSH host
- `user` — SSH user (optional, omit from SSH commands if not set)
- `remote_dir` — project directory on the Pi
- `venv` — virtualenv path on the Pi
- `start_cmd` — command to start the app
- `kill_processes` — list of process names to kill before restart
- `pid_file` — path to PID file on the Pi
- `log_file` — path to main log file on the Pi
- `startup_marker` — string to grep for in logs to confirm startup

Construct the SSH target as: `user@host` if user is set, otherwise just `host`.

## Steps

1. **Kill by PID first.** Check if the PID file exists and kill that specific process:
   ```
   ssh <target> "test -f <pid_file> && kill -9 \$(cat <pid_file>) 2>/dev/null; true"
   ```

2. **Kill orphans.** For each process name in `kill_processes`, run:
   ```
   ssh <target> "killall -9 <process_name> 2>/dev/null; true"
   ```

3. **Start the app.** Run:
   ```
   ssh <target> "cd <remote_dir> && source <venv>/bin/activate && nohup <start_cmd> > /dev/null 2>&1 &"
   ```

4. **Verify startup.** Wait 3 seconds, then run these two checks:

   a. Check PID file exists and process is alive:
   ```
   ssh <target> "test -f <pid_file> && kill -0 \$(cat <pid_file>) 2>/dev/null && echo ALIVE || echo DEAD"
   ```

   b. Get the PID and check for startup marker in the log:
   ```
   ssh <target> "cat <pid_file> 2>/dev/null"
   ssh <target> "grep '<startup_marker>' <log_file> | tail -1"
   ```

5. **Report results.**
   - If ALIVE and startup marker found: report success with PID and startup log line.
   - If DEAD or no startup marker: report failure and show last 20 lines of the log:
     ```
     ssh <target> "tail -20 <log_file>"
     ```

Present results in a compact format:
```
Restart complete:
  PID:     <pid>
  Status:  running
  Startup: <startup marker log line>
```
````

- [ ] **Step 2: Commit**

```bash
cd "C:\Workspace\my-ca\rpi-deploy"
git add commands/restart.md
git commit -m "feat: add /restart command — kill and relaunch on Pi"
```

---

### Task 7: /pi-status command

**Files:**
- Create: `C:\Workspace\my-ca\rpi-deploy\commands\pi-status.md`

- [ ] **Step 1: Write the pi-status command**

Write to `C:\Workspace\my-ca\rpi-deploy\commands\pi-status.md`:

````markdown
---
description: Health check for Raspberry Pi — connectivity, processes, memory, recent logs
allowed-tools: Read, Bash(ssh:*)
---

## Config

Read the `pi-deploy.yaml` file in the current project root to get all connection and path settings. If the file does not exist, stop and tell the user to create one.

Extract these values from the YAML:
- `host` — SSH host
- `user` — SSH user (optional, omit from SSH commands if not set)
- `kill_processes` — list of process names (used to check if they're running)
- `pid_file` — path to PID file on the Pi
- `log_file` — path to main log file on the Pi

Construct the SSH target as: `user@host` if user is set, otherwise just `host`.

## Steps

Run all SSH commands with a 5-second timeout by adding `-o ConnectTimeout=5` to each SSH call.

1. **SSH connectivity check.** Run:
   ```
   ssh -o ConnectTimeout=5 <target> "echo OK"
   ```
   If this fails, report that the Pi is unreachable and stop. Show the error message from SSH.

2. **Gather system info.** Run these commands in a single SSH session to minimize round trips:
   ```
   ssh -o ConnectTimeout=5 <target> "
     echo '---PID---'
     cat <pid_file> 2>/dev/null || echo 'NO_PID_FILE'
     echo '---PID_ALIVE---'
     test -f <pid_file> && kill -0 \$(cat <pid_file>) 2>/dev/null && echo ALIVE || echo DEAD
     echo '---UPTIME---'
     uptime
     echo '---MEMORY---'
     free -m | grep Mem
     echo '---PROCESSES---'
     ps aux | grep -E '<pattern>' | grep -v grep || echo 'NONE'
     echo '---LOGS---'
     tail -5 <log_file> 2>/dev/null || echo 'NO_LOG_FILE'
   "
   ```

   For the `<pattern>` in the PROCESSES section, build a regex alternation from `kill_processes`. For example, if `kill_processes` is `["python", "linphonec"]`, the pattern is `python|linphonec`.

3. **Parse and present results.** Format a compact dashboard from the gathered data:

```
Pi Status: <host>
  SSH:         connected
  App:         <running (pid XXXX) | not running>
  <proc_name>: <running (pid XXXX) | not running>  (one line per kill_processes entry)
  Memory:      <used>/<total> MB
  Load:        <load averages from uptime>
  Last log:    <most recent log line from the 5 fetched>
```

If the app is not running, add a suggestion: "Run `/restart` to start the app."
````

- [ ] **Step 2: Commit**

```bash
cd "C:\Workspace\my-ca\rpi-deploy"
git add commands/pi-status.md
git commit -m "feat: add /pi-status command — Pi health check dashboard"
```

---

### Task 8: Install plugin and end-to-end verification

**Files:**
- Modify: `C:\Workspace\my-ca\yoyo-py\pi-deploy.yaml` (already created in Task 2)

- [ ] **Step 1: Install the plugin in Claude Code**

```bash
claude plugin add "C:\Workspace\my-ca\rpi-deploy"
```

- [ ] **Step 2: Verify all commands appear**

In a Claude Code session, run `/help` and confirm these commands appear under the rpi-deploy plugin:
- `/deploy` — "Git-based deploy to Raspberry Pi (push, pull, restart)"
- `/sync` — "Quick rsync deploy to Raspberry Pi (no commit needed)"
- `/logs` — "Tail application logs from Raspberry Pi"
- `/restart` — "Kill and relaunch the app on Raspberry Pi"
- `/pi-status` — "Health check for Raspberry Pi..."

- [ ] **Step 3: Test /pi-status (requires Pi on network)**

Navigate to the yoyo-py directory and run `/pi-status`. Verify:
- SSH connection succeeds (or fails gracefully with a clear message)
- The dashboard output matches the expected format

- [ ] **Step 4: Test /logs (requires Pi on network)**

Run `/logs 10`. Verify:
- Log lines are returned from the Pi
- Format matches the loguru FILE_FORMAT

Run `/logs --errors`. Verify it reads the error log file.

Run `/logs --filter voip`. Verify it filters by subsystem.

- [ ] **Step 5: Test /restart (requires Pi on network)**

Run `/restart`. Verify:
- Processes are killed
- App restarts
- PID and startup marker are reported

- [ ] **Step 6: Test /deploy (requires Pi on network)**

Make a small commit, then run `/deploy`. Verify:
- Git push succeeds
- Git pull on Pi succeeds
- App restarts with new code

- [ ] **Step 7: Test /sync (requires Pi on network)**

Make an uncommitted change, then run `/sync`. Verify:
- Rsync transfers the changed file
- App restarts

- [ ] **Step 8: Final commit in rpi-deploy repo**

```bash
cd "C:\Workspace\my-ca\rpi-deploy"
git log --oneline
```

Verify the commit history is clean with one commit per feature.

- [ ] **Step 9: Commit yoyo-py config**

```bash
cd "C:\Workspace\my-ca\yoyo-py"
git add pi-deploy.yaml
git commit -m "feat: add pi-deploy.yaml for rpi-deploy plugin"
```
