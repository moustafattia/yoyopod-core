# Slot-Deploy + OTA Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate YoyoPod from in-place `~/yoyopod-core/` deploys to atomic A/B slot deploys under `/opt/yoyopod/`, with an OTA-compatible manifest format and rollback-on-failure, without breaking the current `yoyopod remote` flow.

**Architecture:** On the Pi, immutable versioned release dirs live in `/opt/yoyopod/releases/<version>/`. A `current` symlink points to the active release; systemd runs the app from `/opt/yoyopod/current/bin/launch`. Deploys are: (1) build a packaged release directory on the dev machine, (2) rsync into a new release dir on the Pi, (3) run an offline preflight health probe, (4) atomically flip the symlink, (5) verify liveness. A `previous` symlink + systemd `OnFailure=yoyopod-rollback.service` give automatic rollback. User data lives in `/opt/yoyopod/state/` and is never touched by updates. The release manifest format is already OTA-shaped (signed-artifact placeholder, channel, diff-base, requirements) so a future OTA daemon can plug in without changing the deploy side.

**Tech Stack:** Python 3.12, Typer CLI, `uv`, rsync over SSH, systemd, bash, existing `yoyopod_cli/remote_transport.py` for SSH plumbing. No new runtime dependencies.

---

## Task list (executed by subagents)

1. Release manifest dataclass — `yoyopod_cli/release_manifest.py`
2. Atomic symlink helper — `yoyopod_cli/atomic_symlink.py`
3. Release metadata module (app) — `yoyopod/core/release.py`
4. `yoyopod health` CLI subcommand — `yoyopod_cli/health.py`
5. Build packaging script — `scripts/build_release.py`
6. Slot launcher shell — `deploy/scripts/launch.sh`
7. Pi-side rollback script — `deploy/scripts/rollback.sh`
8. Systemd units — `deploy/systemd/yoyopod-slot.service`, `deploy/systemd/yoyopod-rollback.service`
9. Pi bootstrap script — `deploy/scripts/bootstrap_pi.sh`
10. `yoyopod remote release` CLI — `yoyopod_cli/remote_release.py`
11. SlotPaths dataclass + deploy config — `yoyopod_cli/paths.py`, `deploy/pi-deploy.yaml`
12. Docs + skill update — `docs/SLOT_DEPLOY.md`, `docs/OTA_ROADMAP.md`, `CLAUDE.md`, `skills/yoyopod-deploy/SKILL.md`

Each task is dispatched to a fresh implementer subagent with full task text + TDD requirements + the project's CI gate command (`uv run python scripts/quality.py gate && uv run pytest -q`). After implementation, two-stage review: spec compliance, then code quality.

The full per-task content (test bodies, implementations, exact commands) is held by the controlling agent and provided to each subagent in its dispatch prompt — see the original plan in commit history if needed.

## Self-Review

**Spec coverage:** all 12 elements from the design discussion (target dir layout, manifest, atomic flip, health probes, build packaging, launcher, rollback, systemd, bootstrap, deploy CLI, config extension, docs) have a task.

**Out of scope (explicit):** OTA polling daemon, manifest signing verification, diff-patch apply, RAUC/Mender OS-tier updates. Called out in `docs/OTA_ROADMAP.md` (Task 12).

**Pre-flight per task:** every task ends with `uv run python scripts/quality.py gate && uv run pytest -q` per the repo CI rule in CLAUDE.md.

**Known gaps after this plan:**
1. No sd_notify in the app yet — `StartLimitBurst=3` covers crash-loop detection but `Type=notify` + `WatchdogSec=30` would tighten it. Follow-up.
2. No CI cross-compilation of aarch64 wheels — `scripts/build_release.py` does it on demand. Follow-up.
3. Legacy `yoyopod@.service` is untouched. Separate deprecation PR after migration window.
4. Audit of any hard-coded paths in the app config loader that bypass `YOYOPOD_STATE_DIR`. Bootstrap `--migrate` copies the obvious dirs; deeper audit is its own ticket.
