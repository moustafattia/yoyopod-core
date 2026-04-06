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

   b. Get the PID and check for startup marker matching that PID in the log:
   ```
   ssh <target> "pid=\$(cat <pid_file> 2>/dev/null); echo \$pid; grep '<startup_marker>' <log_file> | tail -1 | grep \"pid=\$pid\""
   ```
   The startup marker must contain `pid=<PID>` to confirm this specific process started (not a stale marker from a previous run).

7. **Report results.**
   - If ALIVE and startup marker with matching PID found: report success with the PID and the startup log line.
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
