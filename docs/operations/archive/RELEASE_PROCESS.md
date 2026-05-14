# Release Process

**Status: paused as of 2026-05-13.**

The Python release pipeline was deleted in Round 0 of the CLI rebuild
([`../../ROADMAP.md`](../../ROADMAP.md)). New prod slot builds
are blocked until Round 3 reintroduces the Rust slot builder, manifest
schema, and preflight tooling. CI's `slot-arm64` and `release` jobs are
disabled (`if: ${{ false }}`) until then.

## What still works

- Previously-shipped slots continue to run on the Pi.
- Reinstalling a previously-shipped slot via SSH +
  `deploy/scripts/install_release.sh` still works (preflight is a no-op
  during the gap; see the `_preflight_slot()` comment in that file).
- The dev lane (`yoyopod target deploy`) is unaffected and remains the
  daily workflow for hardware testing.

## What does NOT work yet

- `yoyopod release {current,bump,set-version,build}` — deleted; not yet
  ported to Rust.
- `scripts/build_release.py` — deleted.
- `deploy/docker/slot-builder.Dockerfile` — deprecated stub.
- `yoyopod target release {push, install-url, status, rollback,
  build-pi}` — not in the Round 1 MVP; returns in Round 3.

## Round 3 sketch

Round 3 reintroduces:

1. A Rust manifest type that mirrors the previous `release_manifest`
   schema (version, channel, artifacts, signatures, requires).
2. A Rust slot-contract module replacing the previous `slot_contract`
   (required dirs, runtime files, config files).
3. A Rust slot builder (replacing `scripts/build_release.py`) that
   produces the same `app/`, `bin/`, `config/`, `venv/`,
   `manifest.json`, and embedded native shims layout.
4. A `yoyopod health {preflight, live}` command bundled inside slots so
   `install_release.sh` can call `<slot>/bin/yoyopod health preflight`.
5. Re-enabled `slot-arm64` and `release` CI jobs.
6. Re-enabled `release.yml` workflow for tag-driven publishing.

## Versioning notes (for when the work resumes)

Semantic versioning: `major` for externally visible contract changes,
`minor` for backward-compatible features, `patch` for fixes. Tag format
`vMAJOR.MINOR.PATCH`.

The version source is no longer in tree (the Python `_version.py` was
deleted with the rest of `yoyopod_cli/`). Round 3 should decide whether
to put the version into `cli/yoyopod/Cargo.toml` only, a plain `VERSION`
file at repo root readable by both CLI and slot builder, or both.
