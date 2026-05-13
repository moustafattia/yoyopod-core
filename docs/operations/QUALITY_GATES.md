# Verification Policy

The project is Rust-only for runtime and CLI. Verification splits
between Rust build/lint checks and Raspberry Pi runtime validation.

## Rust Build Checks

Use focused checks for the crate you changed:

```bash
cargo check --manifest-path device/Cargo.toml -p yoyopod-runtime --locked
cargo check --manifest-path device/Cargo.toml -p yoyopod-ui --locked
cargo check --manifest-path device/Cargo.toml -p yoyopod-media --locked
cargo check --manifest-path device/Cargo.toml -p yoyopod-voip --locked
cargo check --manifest-path device/Cargo.toml -p yoyopod-network --locked
```

For broad workspace changes:

```bash
cargo check --manifest-path device/Cargo.toml --workspace --locked
```

For CLI changes:

```bash
cargo check --manifest-path cli/Cargo.toml
cargo test  --manifest-path cli/Cargo.toml
cargo clippy --manifest-path cli/Cargo.toml --all-targets
```

If native LVGL or Whisplay hardware features are involved, use the
CI-built ARM artifact for the exact commit before claiming hardware
parity.

## Hardware Checks

The Raspberry Pi is the real validation target for the runtime,
display, button input, audio, SIP, modem, power, and LVGL behaviour.

Normal committed-code flow:

```bash
git rev-parse HEAD
yoyopod target mode activate dev
yoyopod target deploy --branch <branch>           # or --sha <commit>
```

Automated on-Pi validation (`yoyopod target validate`) is a Round 1
stub during the CLI rebuild. Until Round 2 restores it, validate
manually after deploy:

```bash
yoyopod target status
yoyopod target logs --follow
journalctl -u yoyopod-dev.service -f
```

Exercise the changed surface on the device with human eyes.

See [`CLI_REBUILD_ROUNDS.md`](CLI_REBUILD_ROUNDS.md).

## Reporting

Always report the checks that actually ran. For hardware validation,
include:

- branch
- exact commit SHA
- artifact names and CI run ID
- Pi command result
- whether the dev service was left running
- manual validation steps performed and their outcomes
