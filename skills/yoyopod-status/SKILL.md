---
name: yoyopod-status
description: Health check for Raspberry Pi lanes, connectivity, processes, memory, recent logs
disable-model-invocation: true
allowed-tools:
  - Read
  - Bash(yoyopod remote:*)
---

## Config

Use `deploy/pi-deploy.yaml` as the shared deploy contract and `deploy/pi-deploy.local.yaml` for machine-specific overrides such as host, SSH user, dev lane checkout, and branch. The tracked dev default is `/opt/yoyopod-dev/checkout`; prod slots live under `/opt/yoyopod-prod`. `yoyopod remote` merges the files directly, and `yoyopod remote config edit` is the preferred way to create or update the local override.

If the file does not exist yet, run `yoyopod remote config edit` first. That command creates `deploy/pi-deploy.local.yaml` automatically before opening it.

## Steps

1. **Check lane ownership first.**
   ```bash
   yoyopod remote mode status
   ```

2. **Run the runtime status command.**
   ```bash
   yoyopod remote status
   ```

3. **Present the result.** Prefer a compact summary with:
   - active lane and any conflict reasons
   - dev service status and prod service status
   - git branch and commit
   - music backend status
   - PiSugar server status
   - PID file state
   - latest startup marker
   - top memory processes

4. **If the app is not running,** explicitly suggest the lane-specific action:
   ```text
   Run `yoyopod remote mode activate dev` for mutable PR testing, or `yoyopod remote mode activate prod` for the packaged slot lane.
   ```
