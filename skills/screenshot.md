---
description: Capture a screenshot of the app's display from Raspberry Pi
allowed-tools: Read, Bash(ssh:*), Bash(scp:*)
argument-hint: "[--readback]"
---

## Config

Read the `pi-deploy.yaml` file in the current project root to get all connection and path settings. If the file does not exist, stop and tell the user to create one.

Extract these values from the YAML:
- `host` — SSH host
- `user` — SSH user (optional, omit from SSH commands if not set)
- `pid_file` — path to PID file on the Pi
- `screenshot_path` — path where the app saves screenshots on the Pi (e.g., `/tmp/yoyopod_screenshot.png`)

Construct the SSH target as: `user@host` if user is set, otherwise just `host`.

## Argument Parsing

Parse the arguments string provided after `/screenshot`:

- **--readback flag:** If `--readback` is present, use SIGUSR2 (LVGL readback — captures what is actually rendered on screen). Otherwise use SIGUSR1 (shadow buffer — captures what the app intended to send to the display).

## Steps

1. **Check the app is running.** Run:
   ```
   ssh <target> "test -f <pid_file> && kill -0 \$(cat <pid_file>) 2>/dev/null && echo ALIVE || echo DEAD"
   ```
   If DEAD, tell the user the app is not running and suggest `/restart`.

2. **Trigger the screenshot.** Send the appropriate signal to the app process:

   For shadow buffer (default, SIGUSR1):
   ```
   ssh <target> "kill -USR1 \$(cat <pid_file>)"
   ```

   For LVGL readback (--readback, SIGUSR2):
   ```
   ssh <target> "kill -USR2 \$(cat <pid_file>)"
   ```

3. **Wait for the screenshot to be written.** Sleep 1 second to allow the app to process the signal and write the PNG file.

4. **Verify the screenshot was created.** Run:
   ```
   ssh <target> "test -f <screenshot_path> && stat -c %Y <screenshot_path> || echo MISSING"
   ```
   If MISSING, report that the screenshot was not created. The app may not support screenshots or the signal handler is not installed.

5. **Copy the screenshot to local machine.** Run:
   ```
   scp <target>:<screenshot_path> <local_temp_path>
   ```
   Use a local temp path like the current directory with a filename of `pi_screenshot.png`.

6. **Display the screenshot.** Use the Read tool to read the local PNG file. Claude is multimodal and can see images directly. Present the screenshot in the conversation.

7. **Explain what was captured.** After showing the image:
   - If default mode: "This is the **shadow buffer** — what the app sent to the display via LVGL flush callbacks."
   - If `--readback`: "This is the **LVGL readback** — what LVGL actually rendered on screen."

   Remind the user they can ask follow-up questions about what they see, such as "why is the status bar missing?" or "what screen is this?"

8. **Clean up.** Delete the local screenshot file after displaying it.
