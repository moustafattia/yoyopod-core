---
name: yoyopod-restart
description: Restart the active YoYoPod dev lane service on Raspberry Pi
disable-model-invocation: true
allowed-tools:
  - Read
  - Bash(yoyopod remote:*)
---

## Config

Use `deploy/pi-deploy.yaml` as the shared deploy contract and `deploy/pi-deploy.local.yaml` for machine-specific overrides such as host, SSH user, dev lane checkout, and branch. `yoyopod remote restart` is a dev-lane helper and expects the selected checkout to match `/opt/yoyopod-dev/checkout`. Prod slots live under `/opt/yoyopod-prod` and should be managed with `yoyopod remote release ...`.

If the file does not exist yet, run `yoyopod remote config edit` first. That command creates `deploy/pi-deploy.local.yaml` automatically before opening it.

## Steps

1. **Check lane ownership first.**
   ```bash
   yoyopod remote mode status
   ```

2. **Restart and verify the dev app service.** Run:
   ```bash
   yoyopod remote restart
   ```

3. **Handle failures.** If the restart fails because prod is active, use `yoyopod remote mode activate dev` before retrying. For other failures, run:
   ```bash
   yoyopod remote logs --lines 20
   ```
   Include the relevant error output in your response.

Report whether the dev lane restart succeeded. For prod, report `yoyopod remote release status` instead of using this skill.
