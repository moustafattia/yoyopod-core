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