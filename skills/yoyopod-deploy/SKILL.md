---
name: yoyopod-deploy
description: Commit-safe Rust-first dev-lane branch/SHA deploy to Raspberry Pi
disable-model-invocation: true
allowed-tools:
  - Read
  - Bash(git status:*)
  - Bash(git branch --show-current:*)
  - Bash(git rev-parse:*)
  - Bash(git push:*)
  - Bash(yoyopod target:*)
---

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

1. **Check local git status.** Run `git status --short`. If there are
   uncommitted or unstaged changes, stop and tell the user: "You have
   uncommitted changes. Commit and push first. The Rust CLI does not
   support dirty-tree deploys (CI-artifact contract)."

2. **Resolve the branch and exact commit.** Run:
   ```bash
   git branch --show-current
   git rev-parse HEAD
   ```
   If the branch is empty, stop and ask the user which branch should be
   deployed.

3. **Push the branch.** Run `git push`. If the branch has no upstream
   yet, run `git push -u origin <branch>`.

4. **Switch the board into the dev lane.** Run:
   ```bash
   yoyopod target mode status
   yoyopod target mode activate dev
   ```
   If `mode status` reports a conflict, include the conflict lines in
   the response and let `mode activate dev` clean the lane before
   deploy.

5. **Deploy the committed branch / SHA.** Run:
   ```bash
   yoyopod target deploy --branch <branch>            # or --sha <commit>
   ```

   `target deploy` automatically:
   - finds the successful CI run for the exact commit
   - downloads `yoyopod-rust-device-arm64-<sha>` via `gh run download`
   - syncs the Pi checkout to the same commit
   - scps + extracts the worker binaries into `device/*/build/`
   - restarts `yoyopod-dev.service` and waits for the startup marker

   Add `--wait-for-ci` if CI is still queued or in-progress.

6. **Optional: stage validation (Round 2 work).** Automated on-Pi
   validation (`yoyopod target validate`) is a stub during the CLI
   rebuild. Validate manually after deploy:
   ```bash
   yoyopod target status
   yoyopod target logs --follow
   ```
   Exercise the changed surface on the device.

7. **Handle failures.** If deploy fails, run:
   ```bash
   yoyopod target logs --lines 20
   ```
   Include the relevant error output in your response.

8. **Report the result.** Include the deployed branch, exact SHA, CI
   run ID, artifact name, Pi host, lane state, and that the dev lane
   was left running for manual testing when the flow succeeded.

## Prod Slot Flow

Prod slot release commands (`yoyopod target release …`) return in Round
3 of the CLI rebuild. Until then, prod releases are paused; see
`docs/ROADMAP.md`. Reinstalling a previously-shipped
slot can still be done manually via SSH + `install_release.sh`.
