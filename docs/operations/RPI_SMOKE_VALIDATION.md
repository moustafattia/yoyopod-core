# Raspberry Pi Smoke Validation

**Status: in transition.**

Automated hardware validation commands (`yoyopod target validate` and
the `yoyopod pi validate …` suite) were deleted in Round 0 of the CLI
rebuild and have not yet been ported back. See
[`CLI_REBUILD_ROUNDS.md`](CLI_REBUILD_ROUNDS.md).

Until Round 2 restores `yoyopod target validate`, validation is split
between local Rust checks and a manual hardware exercise loop.

## 1. Local Rust checks

Run these anywhere:

```bash
cargo check --manifest-path device/Cargo.toml --workspace --locked
cargo test  --manifest-path cli/Cargo.toml
cargo clippy --manifest-path cli/Cargo.toml --all-targets
```

For targeted iteration on a single worker:

```bash
cargo check --manifest-path device/Cargo.toml -p yoyopod-runtime --locked
cargo check --manifest-path device/Cargo.toml -p yoyopod-ui --locked
# … etc
```

If native LVGL or Whisplay hardware features are involved, use the
CI-built ARM artifact (`yoyopod-rust-device-arm64-<sha>`) for the exact
commit under test before claiming hardware parity.

## 2. Deploy and manually exercise on the Pi

```bash
git status --short             # must be clean
git branch --show-current
git rev-parse HEAD
git push
yoyopod target mode activate dev
yoyopod target deploy --branch <branch>         # or --sha <commit>
yoyopod target status
yoyopod target logs --follow                    # leave running while you exercise
```

Use the device:

- power on / off
- navigate the main scene set with the button
- play / pause local music
- attempt an outbound SIP call
- attempt an inbound SIP call (when relevant)
- exercise the cellular path if testing modem changes
- watch for UI freezes, audio dropouts, restart loops, or runtime
  crashes

Take screenshots with `yoyopod target screenshot` (or `--readback` for
the LVGL native path) and compare against design intent.

## 3. What you must report

For every hardware validation, report:

- branch + exact commit SHA
- CI artifact name and run ID
- Pi host
- deploy result (success / failure + stderr)
- manual exercise steps performed
- visible issues found, with timestamps so logs can be cross-referenced
- whether the dev service was left running for further testing

If the change touches a specific worker (UI / media / VoIP / network /
cloud / power / speech), call out that worker's behaviour explicitly
in the report.

## What returns when

| Round | Restores |
|---|---|
| Round 2 | `yoyopod target validate --branch <b> --sha <s>` orchestration |
| Round 2 | Per-stage on-Pi validation entrypoints (deploy / smoke / navigation / stability / voip / cloud-voice / lvgl) |
| Round 3 | Slot install preflight (`yoyopod health preflight`) |
| Round 4+ | Diagnostics (`yoyopod pi voip check`, `pi power battery`, etc.) |

See [`CLI_REBUILD_ROUNDS.md`](CLI_REBUILD_ROUNDS.md).

## Related Docs

- [`PI_DEV_WORKFLOW.md`](PI_DEV_WORKFLOW.md) for the daily deploy loop
- [`DEV_PROD_LANES.md`](DEV_PROD_LANES.md) for lane structure
- [`QUALITY_GATES.md`](QUALITY_GATES.md) for verification policy
- `rules/deploy.md` for the policy that backs hardware validation
