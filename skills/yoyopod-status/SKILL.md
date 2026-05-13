---
name: yoyopod-status
description: Health check for Raspberry Pi lanes, runtime, connectivity, processes, memory, recent logs
disable-model-invocation: true
allowed-tools:
  - Read
  - Bash(yoyopod target:*)
---

## Config

Use `deploy/pi-deploy.yaml` as the shared deploy contract and
`deploy/pi-deploy.local.yaml` for machine-specific overrides such as
host, SSH user, dev lane checkout, and branch. The tracked dev default
is `/opt/yoyopod-dev/checkout`; prod slots live under `/opt/yoyopod-prod`.
`yoyopod target` merges the files directly, and `yoyopod target config
edit` is the preferred way to create or update the local override.

If the file does not exist yet, run `yoyopod target config edit` first.
That command creates `deploy/pi-deploy.local.yaml` automatically before
opening it.

## Steps

1. **Check lane state first.**
   ```bash
   yoyopod target mode status
   ```

2. **Run the runtime status command.**
   ```bash
   yoyopod target status
   ```

3. **Present the result.** Prefer a compact summary with:
   - active lane and any conflict reasons
   - dev service status and prod service status
   - whether `yoyopod-runtime` is running, plus its PID
   - git branch and commit on the Pi checkout
   - Rust artifact presence for runtime/UI/media/VoIP when relevant
   - PID file state
   - latest startup marker
   - top memory processes

4. **If `yoyopod-runtime` is not running,** explicitly suggest the
   lane-specific action:
   ```text
   Run `yoyopod target mode activate dev` for mutable PR testing, or
   `yoyopod target mode activate prod` for the packaged slot lane.
   ```
