# Rust Workspace And Monorepo Migration Design

**Date:** 2026-05-05
**Owner:** Moustafa
**Status:** Draft for review
**Target hardware:** Raspberry Pi Zero 2W, Whisplay dev/prod lanes

---

## 1. Problem

YoYoPod is now Rust-first for the device runtime, but the Rust workspace still
shows its migration history:

- Domain crates are named as `*-host`, while the product architecture now thinks
  in first-class domains: `ui`, `media`, `voip`, `network`, `cloud`, `power`,
  and `speech`.
- Each host repeats worker protocol mechanics: NDJSON envelope structs,
  schema-version checks, ready/error/result helpers, stdin/stdout loops, and
  test harness glue.
- The root repository is starting to become more than a device runtime. Mobile,
  web, SDK, and cloud contract packages will likely join the repo.
- The current layout does not clearly separate device runtime ownership from
  future app/package ownership.

Because the project is still early, it is acceptable to break paths in a
controlled migration if that produces a clearer long-term shape. The migration
must still preserve the deployed process model: `yoyopod-runtime` supervises
domain sidecar binaries over NDJSON stdio.

---

## 2. Goals

- Restructure `yoyopod_rs/` into a cleaner Rust workspace with shared protocol,
  worker, and test-harness crates.
- Keep domain hosts as sidecar worker binaries, not runtime-linked libraries.
- Standardize the worker protocol as one shared NDJSON-over-stdio contract.
- Rename Rust crate directories from `*-host` to domain names.
- Keep shipped binary artifact names explicit, such as `yoyopod-speech-host`,
  even if the source directory becomes `speech/`.
- Prepare the repository for monorepo growth with `apps/` and `packages/`
  roots, without forcing mobile/web packages into the device runtime.
- Update CI, deploy, slot packaging, config defaults, docs, and tests in the
  same migration so the repo is coherent after the cut.

---

## 3. Non-goals

- Do not merge domain workers into the runtime process.
- Do not replace NDJSON stdio with gRPC, HTTP, MQTT, Unix sockets, or a broker.
- Do not move all Python code under `device/` in this migration.
- Do not add the actual mobile or web applications in this migration.
- Do not rewrite Git history as part of this migration.
- Do not change hardware behavior or domain feature behavior.
- Do not rename deployed systemd services in this migration.

---

## 4. Target Layout

The target Rust workspace layout is:

```text
yoyopod_rs/
  Cargo.toml
  Cargo.lock

  runtime/      # orchestrator binary: config, supervision, state, event routing

  protocol/     # shared NDJSON envelope, schema version, encode/decode
  worker/       # shared stdin/stdout loop, ready/error/result helpers
  harness/      # shared test harnesses for worker protocol tests

  cloud/        # cloud worker sidecar binary
  media/        # media worker sidecar binary
  network/      # network worker sidecar binary
  power/        # power worker sidecar binary
  speech/       # speech/Ask worker sidecar binary
  ui/           # UI worker sidecar binary
  voip/         # VoIP worker sidecar binary
```

Future monorepo roots should be introduced at repository root:

```text
apps/
  web/          # future parent/admin web app
  mobile/       # future mobile app

packages/
  contracts/    # API schemas, device/cloud command contracts, generated types
  sdk/          # future TypeScript client package
  ui/           # future shared app UI package, if needed
```

The existing device paths remain at root for now:

```text
yoyopod_rs/     # Rust device runtime and sidecars
yoyopod/        # Python compatibility/runtime surfaces not yet retired
yoyopod_cli/    # Python CLI and Pi operations tooling
deploy/         # install, systemd, slot, and release packaging
config/         # authored device config
```

---

## 5. Dependency Rules

The dependency graph must stay simple:

```text
runtime  -> protocol

worker   -> protocol
harness  -> protocol

cloud    -> protocol + worker
media    -> protocol + worker
network  -> protocol + worker
power    -> protocol + worker
speech   -> protocol + worker
ui       -> protocol + worker
voip     -> protocol + worker
```

Rules:

- `protocol` must not depend on any YoYoPod domain crate.
- `worker` may depend on `protocol`, but must not depend on `runtime`.
- Domain crates must not depend on `runtime`.
- `runtime` must not import domain implementation crates. It supervises worker
  processes through the protocol.
- `harness` is test/dev support only. Production binaries must not need it.
- Future `apps/*` packages must not be dependencies of device runtime crates.
- Future `packages/contracts` may be consumed by apps, cloud tooling, and code
  generators, but must not pull app code back into device runtime crates.

---

## 6. Protocol Contract

The worker protocol remains NDJSON over stdin/stdout. Each line is one JSON
envelope.

Canonical envelope:

```json
{
  "schema_version": 1,
  "kind": "command",
  "type": "voice.ask",
  "request_id": "voice-1",
  "deadline_ms": 12000,
  "timestamp_ms": 1777996800000,
  "payload": {}
}
```

Required fields:

- `schema_version`: integer, currently `1`
- `kind`: `command`, `event`, `result`, `error`, or `heartbeat`
- `type`: namespaced as `<domain>.<action>`
- `payload`: JSON object, defaulting to `{}`

Conditional fields:

- `request_id`: required for command-tied `result` and `error` envelopes
- `deadline_ms`: optional for commands
- `timestamp_ms`: optional sender timestamp

Standard lifecycle messages:

- Worker emits `<domain>.ready` once after startup.
- Runtime sends `<domain>.health`.
- Worker replies with `<domain>.health.result`.
- Runtime sends `<domain>.stop`.
- Worker replies with `<domain>.stopped` or exits cleanly.
- Worker emits `<domain>.error` for async/domain failures.
- Unknown commands return non-retryable `unknown_command`.
- Malformed envelopes return non-retryable `protocol_error`.

---

## 7. Shared Crates

### `protocol`

Owns:

- `SUPPORTED_SCHEMA_VERSION`
- `EnvelopeKind`
- `WorkerEnvelope`
- envelope encode/decode
- schema validation
- constructor helpers:
  - `command`
  - `event`
  - `result`
  - `error`
  - `heartbeat`
- common protocol error type

Does not own:

- domain-specific commands
- runtime state
- worker process supervision
- logging policy

### `worker`

Owns reusable host process mechanics:

- line-based stdin reader loop
- stdout envelope emitter
- ready/error/result convenience helpers
- health/stop boilerplate helpers
- deadline helper primitives where domain-neutral
- graceful shutdown helper behavior

Does not own:

- domain command dispatch tables
- device state machines
- hardware adapters
- cloud providers
- runtime worker supervision

### `harness`

Owns test-only helpers:

- run a worker with fake stdin/stdout
- decode emitted envelopes
- wait for specific envelope types
- assert ready/error/result behavior
- provide shared fixtures for protocol conformance tests

---

## 8. Directory Rename Mapping

Source directories should migrate as:

```text
yoyopod_rs/cloud-host/   -> yoyopod_rs/cloud/
yoyopod_rs/media-host/   -> yoyopod_rs/media/
yoyopod_rs/network-host/ -> yoyopod_rs/network/
yoyopod_rs/power-host/   -> yoyopod_rs/power/
yoyopod_rs/speech-host/  -> yoyopod_rs/speech/
yoyopod_rs/ui-host/      -> yoyopod_rs/ui/
yoyopod_rs/voip-host/    -> yoyopod_rs/voip/
```

Cargo package names should migrate as:

```text
yoyopod-cloud-host   -> yoyopod-cloud
yoyopod-media-host   -> yoyopod-media
yoyopod-network-host -> yoyopod-network
yoyopod-power-host   -> yoyopod-power
yoyopod-speech-host  -> yoyopod-speech
yoyopod-ui-host      -> yoyopod-ui
yoyopod-voip-host    -> yoyopod-voip
```

Binary names should remain deployed-host explicit:

```text
yoyopod-cloud-host
yoyopod-media-host
yoyopod-network-host
yoyopod-power-host
yoyopod-speech-host
yoyopod-ui-host
yoyopod-voip-host
yoyopod-runtime
```

This gives clean source names without making deployed artifacts ambiguous.

---

## 9. Packaging And Config Impact

All path references must be updated together:

- `yoyopod_rs/Cargo.toml` workspace members
- each Rust `Cargo.toml`
- Bazel `BUILD.bazel` files
- CI workflow artifact paths
- `.dockerignore` exceptions for host build outputs
- `yoyopod_cli/slot_contract.py`
- `scripts/build_release.py`
- `yoyopod_cli/build.py`
- `yoyopod_cli/pi/validate/*`
- `config/voice/assistant.yaml`
- Python config defaults under `yoyopod/config/models/`
- Rust runtime config defaults under `yoyopod_rs/runtime/src/config.rs`
- tests that assert paths or artifact names
- docs under `README.md`, `AGENTS.md`, `docs/`, and `rules/`

The slot artifact paths after migration should use the new source directories:

```text
app/yoyopod_rs/cloud/build/yoyopod-cloud-host
app/yoyopod_rs/media/build/yoyopod-media-host
app/yoyopod_rs/network/build/yoyopod-network-host
app/yoyopod_rs/power/build/yoyopod-power-host
app/yoyopod_rs/speech/build/yoyopod-speech-host
app/yoyopod_rs/ui/build/yoyopod-ui-host
app/yoyopod_rs/voip/build/yoyopod-voip-host
app/yoyopod_rs/runtime/build/yoyopod-runtime
```

---

## 10. Migration Slices

### Slice 1: Add Shared Protocol Crate

- Create `yoyopod_rs/protocol`.
- Move one domain's protocol tests to use shared protocol first.
- Keep domain-local protocol modules temporarily as adapters if needed.
- Validate with targeted cargo tests.

Success criteria:

- `protocol` has standalone tests for encode/decode/schema validation.
- At least one host uses `yoyopod-protocol`.
- Runtime can still decode worker envelopes.

### Slice 2: Migrate All Hosts To Shared Protocol

- Replace duplicated `protocol.rs` modules in all hosts.
- Update runtime to use the same crate.
- Delete domain-local envelope definitions.

Success criteria:

- No duplicated `WorkerEnvelope` struct definitions remain outside
  `yoyopod_rs/protocol`.
- All host protocol tests pass.
- Runtime tests pass.

### Slice 3: Add Shared Worker Crate

- Create `yoyopod_rs/worker`.
- Extract common emit, ready, error, health, and stop helpers.
- Migrate the smallest host first, likely `power`.

Success criteria:

- `power` has less protocol-loop boilerplate.
- Behavior stays unchanged.
- Worker tests pass.

### Slice 4: Migrate Remaining Hosts To Shared Worker Helpers

- Move common stdin/stdout loop behavior into `worker`.
- Keep domain dispatch inside each host.
- Normalize ready/error/result behavior across domains.

Success criteria:

- All hosts use `yoyopod-worker` for shared mechanics.
- Domain crates still own domain commands.
- Tests show ready, health, error, stop, and unknown-command behavior for each
  host.

### Slice 5: Add Test Harness Crate

- Create `yoyopod_rs/harness`.
- Move repeated worker test helpers into the harness crate.
- Use it from host integration tests.

Success criteria:

- Worker tests are shorter and consistent.
- Harness is dev/test only.
- Production builds do not depend on `harness`.

### Slice 6: Rename Rust Domain Directories

- Rename `*-host` directories to domain names.
- Update workspace members, package names, Bazel files, CI, deploy, config, docs,
  and tests.
- Keep binary artifact names unchanged.

Success criteria:

- No active references to `yoyopod_rs/*-host/` remain except historical docs.
- `cargo test --manifest-path yoyopod_rs/Cargo.toml --workspace --locked` passes.
- Slot packaging tests pass.
- CLI build tests pass.

### Slice 7: Add Monorepo Roots

- Add empty or documented `apps/` and `packages/` roots.
- Add root documentation explaining ownership boundaries.
- Do not add mobile/web app code yet.

Success criteria:

- Repo has a clear place for future web/mobile packages.
- Device runtime remains independent.
- CI path-filter plan is documented.

---

## 11. Validation Plan

Run after protocol/worker/harness extraction slices:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml --workspace --locked
cargo clippy --manifest-path yoyopod_rs/Cargo.toml --workspace --all-targets --locked -- -D warnings
cargo fmt --manifest-path yoyopod_rs/Cargo.toml --all --check
```

Run after path rename slices:

```bash
uv run pytest -q tests/cli/test_yoyopod_cli_build.py
uv run pytest -q tests/cli/test_slot_contract.py
uv run pytest -q tests/scripts/test_build_release.py
uv run pytest -q tests/deploy/test_ci_workflows.py
uv run pytest -q tests/config/test_config_models.py -k voice
git diff --check
```

Run before hardware validation:

```bash
uv run yoyopod build rust-runtime
uv run yoyopod build voice-worker
```

Pi validation remains exact-commit based:

```bash
yoyopod remote mode activate dev
yoyopod remote sync --branch <branch> --clean-native
yoyopod remote validate --branch <branch> --sha <commit>
```

---

## 12. Risks

- Path churn can break slot packaging if any old `*-host` build path remains.
- Cargo package renames can break Bazel labels and CI artifact discovery.
- Over-extracting `worker` could accidentally centralize domain behavior. Keep
  it limited to process/protocol mechanics.
- Moving Python/device code at the same time would make review too large. Keep
  that as a later migration.
- Future web/mobile packages can pollute device CI if path filters are not set.

---

## 13. Open Decisions

- Whether Cargo package names should become `yoyopod-speech` immediately or keep
  `yoyopod-speech-host` even after source directory rename. This spec recommends
  package names without `-host`, binary names with `-host`.
- Whether `worker` should own a full generic event loop or only small helpers.
  This spec recommends starting with helpers and extracting the loop only after
  two hosts prove the shape.
- Whether `apps/` and `packages/` should be added before or after Rust path
  renames. This spec recommends after the Rust workspace cleanup.

---

## 14. Recommendation

Proceed with the Rust workspace cleanup before adding mobile/web code. The best
first PR is `protocol/` extraction. The best second PR is `worker/` extraction
using `power` as the proving host. Directory renames should come only after the
shared crates are stable, because the rename PR will already touch a broad set
of paths.
