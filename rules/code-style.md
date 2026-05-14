# Code Style

The codebase is Rust-only across the runtime (`device/`) and the
operator CLI (`cli/`). No Python remains as of the Round 0 rebuild
(`docs/ROADMAP.md`).

## Rust (device/ and cli/)

- Stable toolchain, pinned via `rust-toolchain.toml` in each workspace.
- 2021 edition.
- Format with `cargo fmt`; CI uses `--check`.
- Lint with `cargo clippy --all-targets`. Workspace-level lint
  allowlists live in the workspace `Cargo.toml`. Don't suppress lints
  in individual files unless there's a specific reason worth a comment.
- Logging via `tracing` + `tracing-subscriber`. The CLI binary
  initialises the subscriber once at startup; library code uses
  `tracing::{info, warn, error, debug}` directly.
- Errors: `anyhow::Result<T>` in binaries and orchestration code;
  `thiserror`-derived enums when defining a stable error API for a
  library crate.
- No `unsafe` in `cli/` (workspace lint forbids it). `device/` permits
  it only where the LVGL native binding requires it.

## Build commands

```bash
# device runtime + workers
cargo check --manifest-path device/Cargo.toml --workspace --locked
cargo build --manifest-path device/Cargo.toml --release -p yoyopod-runtime

# operator CLI
cargo build --manifest-path cli/Cargo.toml --release
cargo test  --manifest-path cli/Cargo.toml
cargo clippy --manifest-path cli/Cargo.toml --all-targets
```
