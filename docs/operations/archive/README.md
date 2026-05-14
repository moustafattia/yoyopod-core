# Archived Operations Docs

This folder holds operations docs whose subject — prod releases, slot
deploy, OTA, on-Pi automated validation, Python-era profiling — is
**currently paused** while the operator CLI is being rebuilt in Rust.
See [`../../ROADMAP.md`](../../ROADMAP.md).

Each file documents either:
- a contract that's still on disk but has no live CLI commands today
  (slot layout, release flow, OTA wiring), or
- a workflow whose tools were retired (Python profiling helpers,
  `yoyopod pi validate …`).

These docs return to active status when the relevant round of the
rebuild lands. Round 3 reintroduces release/slot tooling; Round 2
restores hardware validation.

| File | Subject | Restored by |
|---|---|---|
| [`RELEASE_PROCESS.md`](RELEASE_PROCESS.md) | Versioning, artifacts, GitHub release workflow | Round 3 |
| [`SLOT_DEPLOY.md`](SLOT_DEPLOY.md) | Prod slot install + rollback flow | Round 3 |
| [`OTA_ROADMAP.md`](OTA_ROADMAP.md) | OTA daemon extension points on top of slot deploy | Round 3 + later |
| [`RPI_SMOKE_VALIDATION.md`](RPI_SMOKE_VALIDATION.md) | Manual + automated on-Pi validation flow | Round 2 |
| [`PI_PROFILING_WORKFLOW.md`](PI_PROFILING_WORKFLOW.md) | Hardware profiling workflow | Round 2 (validation) + ongoing |

For the day-to-day workflow that currently works, see
[`../PI_DEV_WORKFLOW.md`](../PI_DEV_WORKFLOW.md).
