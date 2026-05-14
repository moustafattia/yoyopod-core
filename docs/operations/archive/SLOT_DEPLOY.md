# Slot Deploy (Prod Lane)

**Status: paused as of 2026-05-13.**

The prod slot/OTA CLI commands (`yoyopod target release …`) were
deleted as part of Round 0 of the CLI rebuild and have not been ported
back. New prod slot builds and installs through the CLI are blocked
until Round 3; see
[`../../ROADMAP.md`](../../ROADMAP.md).

This doc documents the slot layout, services, and rollback wiring that
remain in place on previously-bootstrapped Pis. The contract is stable
even while the CLI is being rebuilt.

## Contract (unchanged)

- Prod slots live under `/opt/yoyopod-prod/releases/<version>/`.
- `/opt/yoyopod-prod/current` points at the active release.
- `/opt/yoyopod-prod/previous` points at the rollback release.
- `yoyopod-prod.service` runs `/opt/yoyopod-prod/current/bin/launch`.
- `yoyopod-prod-rollback.service` swaps `current` and `previous` after
  repeated prod failures.
- `deploy/scripts/install_release.sh` installs published artifacts
  directly into `/opt/yoyopod-prod`. The slot preflight step is
  currently a no-op (see comment in that file) until Round 3 restores
  the Rust-based `yoyopod health preflight` command.

## Layout

```text
/opt/yoyopod-prod/
|-- releases/
|   |-- <version>/
|   |   |-- app/
|   |   |-- assets/
|   |   |-- bin/launch
|   |   |-- config/
|   |   |-- manifest.json
|   |   `-- venv/                 (slots predating Round 3 only)
|-- current -> releases/<version>
|-- previous -> releases/<version>
|-- bin/
|   |-- install-release.sh
|   `-- rollback.sh
`-- state/
    `-- tmp/
```

## What works today

- Previously-shipped prod slots continue to run on their boards.
- A previously-shipped artifact can be reinstalled manually via SSH +
  `install_release.sh`:

  ```bash
  ssh <user>@<pi>
  sudo /opt/yoyopod-prod/bin/install-release.sh <path-to-tarball-or-url>
  ```

- Manual rollback:

  ```bash
  ssh <user>@<pi> 'sudo /opt/yoyopod-prod/bin/rollback.sh'
  ```

- Automatic rollback wiring (`OnFailure=yoyopod-prod-rollback.service`)
  is still in `deploy/systemd/yoyopod-prod.service` and unchanged.

## What does NOT work yet

- New slot builds (`yoyopod release build`, `scripts/build_release.py`,
  `deploy/docker/slot-builder.Dockerfile`) — all deleted or deprecated.
  CI `slot-arm64` job disabled.
- All `yoyopod target release …` CLI subcommands (`push`, `rollback`,
  `status`, `install-url`, `build-pi`).
- Structural preflight before slot flip (`yoyopod health preflight`).
- The CI `release` workflow.

## Round 3 sketch

Round 3 reintroduces:

1. A Rust slot-contract module (required dirs, runtime files, config
   files) replacing the old `yoyopod_cli.slot_contract`.
2. A Rust manifest type replacing the old `release_manifest`.
3. A Rust slot builder producing the same on-disk layout as above.
4. `yoyopod health {preflight, live}` as a bundled binary in each slot.
5. `yoyopod target release {push, rollback, status, install-url}` in
   the CLI.
6. Re-enabled CI `slot-arm64` and `release` jobs.

## Related Docs

- [`../../ROADMAP.md`](../../ROADMAP.md)
- [`../DEV_PROD_LANES.md`](../DEV_PROD_LANES.md)
- [`../PI_DEV_WORKFLOW.md`](../PI_DEV_WORKFLOW.md)
- [`RELEASE_PROCESS.md`](RELEASE_PROCESS.md)
