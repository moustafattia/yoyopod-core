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
