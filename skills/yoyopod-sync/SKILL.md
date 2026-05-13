---
name: yoyopod-sync
description: Sync a committed branch / SHA to the Raspberry Pi dev lane
disable-model-invocation: true
allowed-tools:
  - Read
  - Bash(yoyopod target:*)
---

## What this skill does now

This skill is kept for muscle memory. It is a thin wrapper around
`yoyopod target deploy`, which is the committed-code sync command.

The Python-era `yoyopod remote sync --dirty-tree` escape hatch was
deleted in the Round 0 CLI rebuild. The Rust CLI does not support
syncing uncommitted local state — `target deploy` requires a pushed
commit so the matching CI artifact (`yoyopod-rust-device-arm64-<sha>`)
can be downloaded and installed. If you need to test uncommitted work,
commit to a throwaway branch first, push, then deploy.

## Config

Use `deploy/pi-deploy.yaml` as the shared deploy contract and
`deploy/pi-deploy.local.yaml` for machine-specific overrides such as
host, SSH user, and the dev lane checkout. The tracked dev default is
`project_dir: /opt/yoyopod-dev/checkout`; prod slots live under
`/opt/yoyopod-prod`. `yoyopod target` merges the files directly, and
`yoyopod target config edit` is the preferred way to create or update
the local override.

If the file does not exist yet, run `yoyopod target config edit` first.
That command creates `deploy/pi-deploy.local.yaml` automatically before
opening it.

## Steps

1. **Switch the board into the dev lane.** Run:
   ```bash
   yoyopod target mode status
   yoyopod target mode activate dev
   ```

2. **Deploy the committed branch (and optionally exact SHA).** Run:
   ```bash
   yoyopod target deploy --branch <branch>            # current commit
   # or:
   yoyopod target deploy --sha <commit>
   ```

   Add `--wait-for-ci` if the CI run for that commit is still queued or
   in-progress.

   Add `--clean-native` after native LVGL / CMake input changes so the
   Pi-side build dir gets cleared.

3. **Handle failures.** If deploy fails, run:
   ```bash
   yoyopod target logs --lines 20
   ```
   Include the relevant error output in your response.

4. **Report the result clearly.** Include the branch, SHA, CI run ID,
   artifact name, Pi host, and whether `yoyopod-dev.service` came up
   cleanly.
