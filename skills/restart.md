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

   b. Get the PID and check for startup marker matching that PID in the log:
   ```
   ssh <target> "pid=\$(cat <pid_file> 2>/dev/null); echo \$pid; grep '<startup_marker>' <log_file> | tail -1 | grep \"pid=\$pid\""
   ```
   The startup marker must contain `pid=<PID>` to confirm this specific process started (not a stale marker from a previous run).

5. **Report results.**
   - If ALIVE and startup marker with matching PID found: report success with PID and startup log line.
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
