# Rust Workspace Monorepo Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restructure the Rust workspace around shared protocol/worker crates, domain sidecar workers, and future monorepo roots without changing device behavior.

**Architecture:** Keep `yoyopod-runtime` as the orchestrator and keep every domain as a supervised sidecar binary. Extract protocol and process-loop mechanics into shared crates first, then rename domain directories once the behavior is stable. Add `apps/` and `packages/` only after the Rust workspace cut is complete.

**Tech Stack:** Rust Cargo workspace, Bazel BUILD files, Python `uv` CLI/deploy tests, NDJSON over stdin/stdout, GitHub Actions, slot packaging.

---

## Source Spec

Implement this plan against:

- `docs/superpowers/specs/2026-05-05-rust-workspace-and-monorepo-migration-design.md`
- `docs/architecture/WORK_AREAS.md`
- `AGENTS.md`

## Phase Summary

1. `Phase 0`: Commit baseline docs/hygiene and add guard tests.
2. `Phase 1`: Add `yoyopod_rs/protocol` and migrate runtime plus `power` first.
3. `Phase 2`: Migrate every host to `yoyopod-protocol`.
4. `Phase 3`: Add `yoyopod_rs/worker` and extract shared worker helpers.
5. `Phase 4`: Add `yoyopod_rs/harness` and normalize host tests.
6. `Phase 5`: Rename Rust domain directories from `*-host` to domain names.
7. `Phase 6`: Add `apps/` and `packages/` monorepo roots.
8. `Phase 7`: Run full validation and hardware-oriented artifact checks.

Each phase should be independently reviewable. Do not combine Phase 5 with
Phase 1-4 unless the user explicitly asks for a large breaking PR.

---

## Phase 0: Baseline And Guardrails

**Files:**

- Modify: `.gitignore`
- Modify: `README.md`
- Modify: `docs/README.md`
- Modify: `docs/architecture/README.md`
- Create: `docs/architecture/WORK_AREAS.md`
- Create: `docs/superpowers/specs/2026-05-05-rust-workspace-and-monorepo-migration-design.md`
- Create: `docs/superpowers/plans/2026-05-05-rust-workspace-monorepo-migration.md`
- Test: existing `git diff --check`

### Task 0.1: Commit Current Documentation Baseline

- [ ] **Step 1: Inspect current docs/hygiene diff**

Run:

```bash
git status --short --branch
git diff -- .gitignore README.md docs/README.md docs/architecture/README.md docs/architecture/WORK_AREAS.md docs/superpowers/specs/2026-05-05-rust-workspace-and-monorepo-migration-design.md docs/superpowers/plans/2026-05-05-rust-workspace-monorepo-migration.md
```

Expected:

- only documentation and ignore-policy changes are present
- no generated artifacts are staged

- [ ] **Step 2: Verify ignored tracked files are clean**

Run:

```bash
git ls-files -ci --exclude-standard
git clean -ndX
```

Expected:

- `git ls-files -ci --exclude-standard` prints nothing
- `git clean -ndX` prints nothing

- [ ] **Step 3: Run whitespace check**

Run:

```bash
git diff --check
```

Expected:

- exit code `0`
- no whitespace errors

- [ ] **Step 4: Commit the baseline**

Run:

```bash
git add .gitignore README.md docs/README.md docs/architecture/README.md docs/architecture/WORK_AREAS.md docs/superpowers/specs/2026-05-05-rust-workspace-and-monorepo-migration-design.md docs/superpowers/plans/2026-05-05-rust-workspace-monorepo-migration.md
git commit -m "docs: define rust workspace migration plan"
```

Expected:

- one docs-only commit exists
- working tree is clean

### Task 0.2: Add Workspace Structure Guard Tests

**Files:**

- Create: `tests/cli/test_rust_workspace_structure.py`

- [ ] **Step 1: Write failing guard tests**

Create `tests/cli/test_rust_workspace_structure.py`:

```python
from __future__ import annotations

from pathlib import Path

import tomllib


REPO_ROOT = Path(__file__).resolve().parents[2]
RUST_ROOT = REPO_ROOT / "yoyopod_rs"


def _cargo_workspace_members() -> set[str]:
    with (RUST_ROOT / "Cargo.toml").open("rb") as handle:
        payload = tomllib.load(handle)
    return set(payload["workspace"]["members"])


def test_rust_workspace_has_no_generated_target_member() -> None:
    members = _cargo_workspace_members()

    assert "target" not in members
    assert not any(member.startswith("target/") for member in members)


def test_rust_workspace_tracks_lockfile_even_when_cargo_lock_is_ignored() -> None:
    assert (RUST_ROOT / "Cargo.lock").is_file()
```

- [ ] **Step 2: Run the guard tests**

Run:

```bash
uv run pytest -q tests/cli/test_rust_workspace_structure.py
```

Expected:

- tests pass on the current repo

- [ ] **Step 3: Commit guard tests**

Run:

```bash
git add tests/cli/test_rust_workspace_structure.py
git commit -m "test: guard rust workspace structure"
```

Expected:

- second small commit exists

---

## Phase 1: Shared Protocol Crate With Runtime And Power First

**Files:**

- Create: `yoyopod_rs/protocol/Cargo.toml`
- Create: `yoyopod_rs/protocol/src/lib.rs`
- Create: `yoyopod_rs/protocol/tests/envelope.rs`
- Modify: `yoyopod_rs/Cargo.toml`
- Modify: `yoyopod_rs/runtime/Cargo.toml`
- Modify: `yoyopod_rs/runtime/src/protocol.rs`
- Modify: `yoyopod_rs/power-host/Cargo.toml`
- Modify: `yoyopod_rs/power-host/src/protocol.rs`
- Modify: `yoyopod_rs/power-host/tests/protocol.rs` if present
- Modify: `yoyopod_rs/power-host/tests/worker.rs`

### Task 1.1: Create Protocol Crate Tests

- [ ] **Step 1: Add protocol crate to workspace**

Modify `yoyopod_rs/Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "cloud-host",
    "media-host",
    "network-host",
    "power-host",
    "protocol",
    "runtime",
    "speech-host",
    "ui-host",
    "voip-host",
]
```

- [ ] **Step 2: Create `yoyopod_rs/protocol/Cargo.toml`**

```toml
[package]
name = "yoyopod-protocol"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "2.0"
```

- [ ] **Step 3: Write protocol tests first**

Create `yoyopod_rs/protocol/tests/envelope.rs`:

```rust
use serde_json::json;
use yoyopod_protocol::{EnvelopeKind, WorkerEnvelope, SUPPORTED_SCHEMA_VERSION};

#[test]
fn command_round_trip_preserves_required_fields() {
    let encoded = WorkerEnvelope::command(
        "power.health",
        Some("req-1".to_string()),
        json!({"probe": true}),
    )
    .encode()
    .expect("encode command");

    assert!(encoded.ends_with(b"\n"));

    let decoded = WorkerEnvelope::decode(&encoded).expect("decode command");
    assert_eq!(decoded.schema_version, SUPPORTED_SCHEMA_VERSION);
    assert_eq!(decoded.kind, EnvelopeKind::Command);
    assert_eq!(decoded.message_type, "power.health");
    assert_eq!(decoded.request_id.as_deref(), Some("req-1"));
    assert_eq!(decoded.payload["probe"], true);
}

#[test]
fn result_constructor_preserves_request_id() {
    let envelope = WorkerEnvelope::result(
        "power.health.result",
        Some("req-1".to_string()),
        json!({"healthy": true}),
    );

    assert_eq!(envelope.kind, EnvelopeKind::Result);
    assert_eq!(envelope.request_id.as_deref(), Some("req-1"));
    assert_eq!(envelope.payload["healthy"], true);
}

#[test]
fn error_constructor_uses_standard_payload() {
    let envelope = WorkerEnvelope::error(
        Some("req-1".to_string()),
        "invalid_payload",
        "battery value must be numeric",
        false,
    );

    assert_eq!(envelope.kind, EnvelopeKind::Error);
    assert_eq!(envelope.message_type, "worker.error");
    assert_eq!(envelope.payload["code"], "invalid_payload");
    assert_eq!(envelope.payload["message"], "battery value must be numeric");
    assert_eq!(envelope.payload["retryable"], false);
}

#[test]
fn decode_rejects_unsupported_schema_version() {
    let error = WorkerEnvelope::decode(
        br#"{"schema_version":999,"kind":"command","type":"power.health","payload":{}}"#,
    )
    .expect_err("schema version must be rejected");

    assert!(error.to_string().contains("unsupported"));
}

#[test]
fn decode_rejects_empty_type() {
    let error = WorkerEnvelope::decode(
        br#"{"schema_version":1,"kind":"command","type":"","payload":{}}"#,
    )
    .expect_err("empty type must be rejected");

    assert!(error.to_string().contains("type"));
}
```

- [ ] **Step 4: Run tests and confirm they fail**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-protocol --locked
```

Expected:

- fails because `yoyopod_rs/protocol/src/lib.rs` does not yet define the types

### Task 1.2: Implement Protocol Crate

- [ ] **Step 1: Create `yoyopod_rs/protocol/src/lib.rs`**

```rust
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvelopeKind {
    Command,
    Event,
    Result,
    Error,
    Heartbeat,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WorkerEnvelope {
    pub schema_version: u32,
    pub kind: EnvelopeKind,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deadline_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp_ms: Option<u64>,
    #[serde(default = "empty_payload")]
    pub payload: Value,
}

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("invalid JSON worker envelope: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error("unsupported schema_version {actual}; expected {expected}")]
    UnsupportedSchemaVersion { actual: u32, expected: u32 },
    #[error("worker envelope type must be a non-empty string")]
    EmptyMessageType,
    #[error("worker envelope payload must be a JSON object")]
    InvalidPayload,
}

impl WorkerEnvelope {
    pub fn decode(line: &[u8]) -> Result<Self, ProtocolError> {
        let mut envelope: Self = serde_json::from_slice(line)?;
        envelope.validate()?;
        if envelope.payload.is_null() {
            envelope.payload = empty_payload();
        }
        Ok(envelope)
    }

    pub fn encode(&self) -> Result<Vec<u8>, ProtocolError> {
        self.validate()?;
        let mut encoded = serde_json::to_vec(self)?;
        encoded.push(b'\n');
        Ok(encoded)
    }

    pub fn command(message_type: &str, request_id: Option<String>, payload: Value) -> Self {
        Self::new(EnvelopeKind::Command, message_type, request_id, payload)
    }

    pub fn event(message_type: &str, payload: Value) -> Self {
        Self::new(EnvelopeKind::Event, message_type, None, payload)
    }

    pub fn result(message_type: &str, request_id: Option<String>, payload: Value) -> Self {
        Self::new(EnvelopeKind::Result, message_type, request_id, payload)
    }

    pub fn error(
        request_id: Option<String>,
        code: &str,
        message: impl Into<String>,
        retryable: bool,
    ) -> Self {
        Self::new(
            EnvelopeKind::Error,
            "worker.error",
            request_id,
            json!({
                "code": code,
                "message": message.into(),
                "retryable": retryable,
            }),
        )
    }

    pub fn heartbeat(message_type: &str, payload: Value) -> Self {
        Self::new(EnvelopeKind::Heartbeat, message_type, None, payload)
    }

    fn new(
        kind: EnvelopeKind,
        message_type: &str,
        request_id: Option<String>,
        payload: Value,
    ) -> Self {
        Self {
            schema_version: SUPPORTED_SCHEMA_VERSION,
            kind,
            message_type: message_type.to_string(),
            request_id,
            deadline_ms: None,
            timestamp_ms: None,
            payload,
        }
    }

    fn validate(&self) -> Result<(), ProtocolError> {
        if self.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(ProtocolError::UnsupportedSchemaVersion {
                actual: self.schema_version,
                expected: SUPPORTED_SCHEMA_VERSION,
            });
        }
        if self.message_type.trim().is_empty() {
            return Err(ProtocolError::EmptyMessageType);
        }
        if !self.payload.is_object() && !self.payload.is_null() {
            return Err(ProtocolError::InvalidPayload);
        }
        Ok(())
    }
}

fn empty_payload() -> Value {
    json!({})
}
```

- [ ] **Step 2: Run protocol tests**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-protocol --locked
```

Expected:

- tests pass

### Task 1.3: Migrate Runtime And Power To Shared Protocol

- [ ] **Step 1: Add dependency to runtime and power**

Modify `yoyopod_rs/runtime/Cargo.toml` and `yoyopod_rs/power-host/Cargo.toml`:

```toml
yoyopod-protocol = { path = "../protocol" }
```

- [ ] **Step 2: Replace runtime protocol module with re-export**

Replace `yoyopod_rs/runtime/src/protocol.rs` with:

```rust
pub use yoyopod_protocol::{
    EnvelopeKind, ProtocolError, WorkerEnvelope, SUPPORTED_SCHEMA_VERSION,
};
```

- [ ] **Step 3: Replace power protocol module with re-export**

Replace `yoyopod_rs/power-host/src/protocol.rs` with:

```rust
pub use yoyopod_protocol::{
    EnvelopeKind, ProtocolError, WorkerEnvelope, SUPPORTED_SCHEMA_VERSION,
};
```

- [ ] **Step 4: Update imports if names differ**

Run:

```bash
rg -n "ProtocolError|EnvelopeKind|WorkerEnvelope|SUPPORTED_SCHEMA_VERSION" yoyopod_rs/runtime yoyopod_rs/power-host -g "*.rs"
```

If compile errors show old error variant names, update local matches to the
new shared names:

```rust
ProtocolError::InvalidJson(error)
ProtocolError::UnsupportedSchemaVersion { actual, expected }
ProtocolError::EmptyMessageType
ProtocolError::InvalidPayload
```

- [ ] **Step 5: Run targeted tests**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-runtime -p yoyopod-power-host -p yoyopod-protocol --locked
```

Expected:

- runtime, power, and protocol tests pass

- [ ] **Step 6: Commit Phase 1**

Run:

```bash
git add yoyopod_rs/Cargo.toml yoyopod_rs/Cargo.lock yoyopod_rs/protocol yoyopod_rs/runtime yoyopod_rs/power-host
git commit -m "refactor(rust): add shared worker protocol crate"
```

---

## Phase 2: Migrate All Hosts To Shared Protocol

**Files:**

- Modify: `yoyopod_rs/cloud-host/Cargo.toml`
- Modify: `yoyopod_rs/media-host/Cargo.toml`
- Modify: `yoyopod_rs/network-host/Cargo.toml`
- Modify: `yoyopod_rs/speech-host/Cargo.toml`
- Modify: `yoyopod_rs/ui-host/Cargo.toml`
- Modify: `yoyopod_rs/voip-host/Cargo.toml`
- Modify: each host `src/protocol.rs`
- Modify: host tests using local protocol types

### Task 2.1: Convert Remaining Host Protocol Modules To Re-exports

- [ ] **Step 1: Add dependency to every host**

For each host `Cargo.toml`, add:

```toml
yoyopod-protocol = { path = "../protocol" }
```

Hosts:

```text
yoyopod_rs/cloud-host/Cargo.toml
yoyopod_rs/media-host/Cargo.toml
yoyopod_rs/network-host/Cargo.toml
yoyopod_rs/speech-host/Cargo.toml
yoyopod_rs/ui-host/Cargo.toml
yoyopod_rs/voip-host/Cargo.toml
```

- [ ] **Step 2: Replace each `src/protocol.rs`**

Use this exact body unless a host has an extra domain-specific helper:

```rust
pub use yoyopod_protocol::{
    EnvelopeKind, ProtocolError, WorkerEnvelope, SUPPORTED_SCHEMA_VERSION,
};
```

If a host has domain-specific helper functions in `protocol.rs`, move those
helpers into `worker.rs` or a new domain file before replacing the envelope
definitions.

- [ ] **Step 3: Run compile to expose mismatches**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml --workspace --locked
```

Expected:

- first run may fail with variant/import mismatches
- no behavior assertions should need to change

- [ ] **Step 4: Fix compile mismatches only**

Use `rg` to find old protocol definitions:

```bash
rg -n "pub struct WorkerEnvelope|enum ProtocolError|SUPPORTED_SCHEMA_VERSION|EnvelopeKind" yoyopod_rs -g "*.rs" -g "!target"
```

Expected after fixes:

- `pub struct WorkerEnvelope` appears only in `yoyopod_rs/protocol/src/lib.rs`
- host `src/protocol.rs` files are re-exports

- [ ] **Step 5: Run all Rust tests**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml --workspace --locked
```

Expected:

- all Rust workspace tests pass

- [ ] **Step 6: Commit Phase 2**

Run:

```bash
git add yoyopod_rs
git commit -m "refactor(rust): share worker protocol across hosts"
```

---

## Phase 3: Shared Worker Crate

**Files:**

- Create: `yoyopod_rs/worker/Cargo.toml`
- Create: `yoyopod_rs/worker/src/lib.rs`
- Create: `yoyopod_rs/worker/tests/io.rs`
- Modify: `yoyopod_rs/Cargo.toml`
- Modify: `yoyopod_rs/power-host/Cargo.toml`
- Modify: `yoyopod_rs/power-host/src/worker.rs`

### Task 3.1: Add Worker Crate With Emit And Standard Envelopes

- [ ] **Step 1: Add `worker` member**

Modify `yoyopod_rs/Cargo.toml`:

```toml
members = [
    "cloud-host",
    "media-host",
    "network-host",
    "power-host",
    "protocol",
    "runtime",
    "speech-host",
    "ui-host",
    "voip-host",
    "worker",
]
```

- [ ] **Step 2: Create `yoyopod_rs/worker/Cargo.toml`**

```toml
[package]
name = "yoyopod-worker"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
serde_json = "1.0"
yoyopod-protocol = { path = "../protocol" }
```

- [ ] **Step 3: Write tests for emit and lifecycle helpers**

Create `yoyopod_rs/worker/tests/io.rs`:

```rust
use serde_json::json;
use yoyopod_protocol::WorkerEnvelope;
use yoyopod_worker::{emit, ready_event, standard_error};

#[test]
fn emit_writes_one_newline_delimited_envelope() {
    let mut output = Vec::new();
    emit(
        &mut output,
        &WorkerEnvelope::event("power.ready", json!({"ready": true})),
    )
    .expect("emit");

    let rendered = String::from_utf8(output).expect("utf8");
    assert!(rendered.ends_with('\n'));
    assert!(rendered.contains(r#""type":"power.ready""#));
}

#[test]
fn ready_event_uses_domain_namespace() {
    let envelope = ready_event("power", json!({"ready": true}));

    assert_eq!(envelope.message_type, "power.ready");
    assert_eq!(envelope.payload["ready"], true);
}

#[test]
fn standard_error_uses_domain_namespace() {
    let envelope = standard_error(
        "power",
        Some("req-1".to_string()),
        "invalid_payload",
        "bad battery payload",
        false,
    );

    assert_eq!(envelope.message_type, "power.error");
    assert_eq!(envelope.request_id.as_deref(), Some("req-1"));
    assert_eq!(envelope.payload["code"], "invalid_payload");
    assert_eq!(envelope.payload["retryable"], false);
}
```

- [ ] **Step 4: Implement `worker/src/lib.rs`**

Create `yoyopod_rs/worker/src/lib.rs`:

```rust
use std::io::Write;

use anyhow::Result;
use serde_json::Value;
use yoyopod_protocol::{EnvelopeKind, WorkerEnvelope};

pub fn emit<W>(output: &mut W, envelope: &WorkerEnvelope) -> Result<()>
where
    W: Write,
{
    output.write_all(&envelope.encode()?)?;
    output.flush()?;
    Ok(())
}

pub fn ready_event(domain: &str, payload: Value) -> WorkerEnvelope {
    WorkerEnvelope::event(&format!("{domain}.ready"), payload)
}

pub fn health_result(
    domain: &str,
    request_id: Option<String>,
    payload: Value,
) -> WorkerEnvelope {
    WorkerEnvelope::result(&format!("{domain}.health.result"), request_id, payload)
}

pub fn stopped_event(domain: &str, payload: Value) -> WorkerEnvelope {
    WorkerEnvelope::event(&format!("{domain}.stopped"), payload)
}

pub fn standard_error(
    domain: &str,
    request_id: Option<String>,
    code: &str,
    message: impl Into<String>,
    retryable: bool,
) -> WorkerEnvelope {
    let mut envelope = WorkerEnvelope::error(request_id, code, message, retryable);
    envelope.kind = EnvelopeKind::Error;
    envelope.message_type = format!("{domain}.error");
    envelope
}
```

- [ ] **Step 5: Run worker tests**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-worker --locked
```

Expected:

- tests pass

### Task 3.2: Migrate Power Host To Worker Helpers

- [ ] **Step 1: Add power dependency**

Modify `yoyopod_rs/power-host/Cargo.toml`:

```toml
yoyopod-worker = { path = "../worker" }
```

- [ ] **Step 2: Replace local emit helper in power worker**

In `yoyopod_rs/power-host/src/worker.rs`, replace local `emit` logic with:

```rust
use yoyopod_worker::{emit, health_result, ready_event, standard_error, stopped_event};
```

Use:

```rust
emit(output, &ready_event("power", json!({"ready": true})))?;
```

For errors, use:

```rust
emit(
    output,
    &standard_error("power", request_id, "invalid_payload", error.to_string(), false),
)?;
```

- [ ] **Step 3: Run power tests**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-power-host -p yoyopod-worker --locked
```

Expected:

- power behavior tests pass unchanged

- [ ] **Step 4: Commit Phase 3**

Run:

```bash
git add yoyopod_rs/Cargo.toml yoyopod_rs/Cargo.lock yoyopod_rs/worker yoyopod_rs/power-host
git commit -m "refactor(rust): add shared worker helpers"
```

---

## Phase 4: Shared Harness Crate

**Files:**

- Create: `yoyopod_rs/harness/Cargo.toml`
- Create: `yoyopod_rs/harness/src/lib.rs`
- Modify: `yoyopod_rs/Cargo.toml`
- Modify: host `Cargo.toml` dev-dependencies
- Modify: host `tests/worker.rs`

### Task 4.1: Add Harness Crate

- [ ] **Step 1: Add `harness` workspace member**

Modify `yoyopod_rs/Cargo.toml`:

```toml
members = [
    "cloud-host",
    "harness",
    "media-host",
    "network-host",
    "power-host",
    "protocol",
    "runtime",
    "speech-host",
    "ui-host",
    "voip-host",
    "worker",
]
```

- [ ] **Step 2: Create `yoyopod_rs/harness/Cargo.toml`**

```toml
[package]
name = "yoyopod-harness"
version = "0.1.0"
edition = "2021"

[dependencies]
serde_json = "1.0"
yoyopod-protocol = { path = "../protocol" }
```

- [ ] **Step 3: Create `yoyopod_rs/harness/src/lib.rs`**

```rust
use serde_json::Value;
use yoyopod_protocol::WorkerEnvelope;

pub fn decode_envelopes(output: &[u8]) -> Vec<WorkerEnvelope> {
    String::from_utf8_lossy(output)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| WorkerEnvelope::decode(line.as_bytes()).expect("decode worker envelope"))
        .collect()
}

pub fn decode_values(output: &[u8]) -> Vec<Value> {
    String::from_utf8_lossy(output)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("decode worker JSON line"))
        .collect()
}

pub fn find_envelope<'a>(
    envelopes: &'a [WorkerEnvelope],
    message_type: &str,
) -> &'a WorkerEnvelope {
    envelopes
        .iter()
        .find(|envelope| envelope.message_type == message_type)
        .unwrap_or_else(|| panic!("missing envelope type {message_type}"))
}

pub fn find_value<'a>(values: &'a [Value], message_type: &str) -> &'a Value {
    values
        .iter()
        .find(|value| value["type"] == message_type)
        .unwrap_or_else(|| panic!("missing JSON envelope type {message_type}"))
}
```

- [ ] **Step 4: Run harness tests through compile**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-harness --locked
```

Expected:

- crate compiles

### Task 4.2: Migrate Worker Tests To Harness

- [ ] **Step 1: Add dev-dependency to host crates**

For each host with worker tests, add:

```toml
[dev-dependencies]
yoyopod-harness = { path = "../harness" }
```

Preserve existing dev-dependencies.

- [ ] **Step 2: Replace duplicated decode helpers**

In each `tests/worker.rs`, replace local decode/find helpers with:

```rust
use yoyopod_harness::{decode_envelopes, decode_values, find_envelope, find_value};
```

Use `decode_envelopes` where tests assert `WorkerEnvelope` fields. Use
`decode_values` where tests assert raw JSON fields.

- [ ] **Step 3: Run all host worker tests**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml -p yoyopod-cloud-host -p yoyopod-media-host -p yoyopod-network-host -p yoyopod-power-host -p yoyopod-speech-host -p yoyopod-ui-host -p yoyopod-voip-host --test worker --locked
```

Expected:

- all host worker tests pass

- [ ] **Step 4: Commit Phase 4**

Run:

```bash
git add yoyopod_rs
git commit -m "test(rust): share worker harness helpers"
```

---

## Phase 5: Rename Rust Domain Directories

**Files:**

- Move: `yoyopod_rs/cloud-host` to `yoyopod_rs/cloud`
- Move: `yoyopod_rs/media-host` to `yoyopod_rs/media`
- Move: `yoyopod_rs/network-host` to `yoyopod_rs/network`
- Move: `yoyopod_rs/power-host` to `yoyopod_rs/power`
- Move: `yoyopod_rs/speech-host` to `yoyopod_rs/speech`
- Move: `yoyopod_rs/ui-host` to `yoyopod_rs/ui`
- Move: `yoyopod_rs/voip-host` to `yoyopod_rs/voip`
- Modify: `.github/workflows/ci.yml`
- Modify: `.dockerignore`
- Modify: `config/voice/assistant.yaml`
- Modify: `scripts/build_release.py`
- Modify: `yoyopod_cli/build.py`
- Modify: `yoyopod_cli/slot_contract.py`
- Modify: `yoyopod_cli/pi/validate/cloud_voice.py`
- Modify: `yoyopod/config/models/voice.py`
- Modify: `yoyopod_rs/runtime/src/config.rs`
- Modify: docs/tests with path references

### Task 5.1: Move Directories And Update Cargo Workspace

- [ ] **Step 1: Move directories with Git**

Run:

```bash
git mv yoyopod_rs/cloud-host yoyopod_rs/cloud
git mv yoyopod_rs/media-host yoyopod_rs/media
git mv yoyopod_rs/network-host yoyopod_rs/network
git mv yoyopod_rs/power-host yoyopod_rs/power
git mv yoyopod_rs/speech-host yoyopod_rs/speech
git mv yoyopod_rs/ui-host yoyopod_rs/ui
git mv yoyopod_rs/voip-host yoyopod_rs/voip
```

Expected:

- Git records renames, not delete/add churn where possible

- [ ] **Step 2: Update workspace members**

Modify `yoyopod_rs/Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "cloud",
    "harness",
    "media",
    "network",
    "power",
    "protocol",
    "runtime",
    "speech",
    "ui",
    "voip",
    "worker",
]
```

- [ ] **Step 3: Update path dependencies**

Run:

```bash
rg -n "\\.\\./(cloud-host|media-host|network-host|power-host|speech-host|ui-host|voip-host)|yoyopod_rs/(cloud-host|media-host|network-host|power-host|speech-host|ui-host|voip-host)" yoyopod_rs -g "Cargo.toml" -g "*.rs" -g "BUILD.bazel"
```

Replace old paths with new paths:

```text
../protocol stays ../protocol
../worker stays ../worker
../harness stays ../harness
yoyopod_rs/speech-host becomes yoyopod_rs/speech
```

- [ ] **Step 4: Run cargo metadata**

Run:

```bash
cargo metadata --manifest-path yoyopod_rs/Cargo.toml --locked --no-deps
```

Expected:

- command succeeds

### Task 5.2: Rename Package Names But Preserve Binary Names

- [ ] **Step 1: Update package names**

In each domain `Cargo.toml`, set:

```toml
name = "yoyopod-cloud"
name = "yoyopod-media"
name = "yoyopod-network"
name = "yoyopod-power"
name = "yoyopod-speech"
name = "yoyopod-ui"
name = "yoyopod-voip"
```

- [ ] **Step 2: Preserve binary names**

If a crate has a `[[bin]]` section, keep or add:

```toml
[[bin]]
name = "yoyopod-speech-host"
path = "src/main.rs"
```

Use the matching binary name for each domain:

```text
yoyopod-cloud-host
yoyopod-media-host
yoyopod-network-host
yoyopod-power-host
yoyopod-speech-host
yoyopod-ui-host
yoyopod-voip-host
```

- [ ] **Step 3: Update Rust crate import names**

Run:

```bash
rg -n "yoyopod_(cloud|media|network|power|speech|ui|voip)_host" yoyopod_rs -g "*.rs"
```

Replace imports:

```text
yoyopod_cloud_host   -> yoyopod_cloud
yoyopod_media_host   -> yoyopod_media
yoyopod_network_host -> yoyopod_network
yoyopod_power_host   -> yoyopod_power
yoyopod_speech_host  -> yoyopod_speech
yoyopod_ui_host      -> yoyopod_ui
yoyopod_voip_host    -> yoyopod_voip
```

- [ ] **Step 4: Run cargo check**

Run:

```bash
cargo check --manifest-path yoyopod_rs/Cargo.toml --workspace --locked
```

Expected:

- compile succeeds after import updates

### Task 5.3: Update Build, Slot, Config, CI Paths

- [ ] **Step 1: Replace source path references**

Run:

```bash
rg -n "cloud-host|media-host|network-host|power-host|speech-host|ui-host|voip-host" . -g "!yoyopod_rs/target/**"
```

Update active references in:

```text
.github/workflows/ci.yml
.dockerignore
config/voice/assistant.yaml
deploy/docker/slot-builder.Dockerfile
scripts/build_release.py
tests/
yoyopod/config/models/voice.py
yoyopod_cli/build.py
yoyopod_cli/pi/validate/cloud_voice.py
yoyopod_cli/slot_contract.py
yoyopod_rs/runtime/src/config.rs
yoyopod_rs/runtime/tests/config.rs
```

Expected path examples:

```text
yoyopod_rs/speech/build/yoyopod-speech-host
app/yoyopod_rs/speech/build/yoyopod-speech-host
yoyopod_rs/runtime/build/yoyopod-runtime
```

- [ ] **Step 2: Keep historical docs unchanged only when clearly historical**

Historical docs under `docs/archive/`, `docs/history/`, and older
`docs/superpowers/` plans may keep old names if the text describes past work.
Current docs and tests must use new names.

- [ ] **Step 3: Run targeted Python tests**

Run:

```bash
uv run pytest -q tests/cli/test_yoyopod_cli_build.py
uv run pytest -q tests/cli/test_slot_contract.py
uv run pytest -q tests/scripts/test_build_release.py
uv run pytest -q tests/deploy/test_ci_workflows.py
uv run pytest -q tests/config/test_config_models.py -k voice
```

Expected:

- all targeted packaging/config tests pass

### Task 5.4: Validate Full Rust Rename

- [ ] **Step 1: Run full Rust workspace tests**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml --workspace --locked
```

Expected:

- all Rust tests pass

- [ ] **Step 2: Run clippy and fmt**

Run:

```bash
cargo clippy --manifest-path yoyopod_rs/Cargo.toml --workspace --all-targets --locked -- -D warnings
cargo fmt --manifest-path yoyopod_rs/Cargo.toml --all --check
```

Expected:

- both commands pass

- [ ] **Step 3: Ensure no active old source paths remain**

Run:

```bash
rg -n "yoyopod_rs/(cloud-host|media-host|network-host|power-host|speech-host|ui-host|voip-host)|yoyopod_(cloud|media|network|power|speech|ui|voip)_host" . -g "!docs/archive/**" -g "!docs/history/**" -g "!docs/superpowers/**" -g "!yoyopod_rs/target/**"
```

Expected:

- no output

- [ ] **Step 4: Commit Phase 5**

Run:

```bash
git add .github .dockerignore config deploy scripts tests yoyopod yoyopod_cli yoyopod_rs README.md docs AGENTS.md rules
git commit -m "refactor(rust): rename host workspace directories"
```

---

## Phase 6: Add Monorepo Roots

**Files:**

- Create: `apps/README.md`
- Create: `apps/.gitkeep`
- Create: `packages/README.md`
- Create: `packages/.gitkeep`
- Modify: `README.md`
- Modify: `docs/README.md`
- Modify: `docs/architecture/WORK_AREAS.md`
- Modify: `.gitignore`

### Task 6.1: Add Root Folders Without App Code

- [ ] **Step 1: Create `apps/README.md`**

```markdown
# Apps

Future user-facing applications live here.

- `web/` will hold the parent/admin web app when it exists.
- `mobile/` will hold the mobile app when it exists.

Apps may depend on packages under `packages/`, especially contracts and SDKs.
Apps must not be dependencies of the device runtime.
```

- [ ] **Step 2: Create `packages/README.md`**

```markdown
# Packages

Shared monorepo packages live here.

Planned packages:

- `contracts/` for API schemas, device/cloud command contracts, and generated types.
- `sdk/` for client libraries used by apps and tooling.
- `ui/` for shared app UI components if web and mobile need them.

Device runtime crates under `yoyopod_rs/` must not import app packages.
```

- [ ] **Step 3: Add `.gitkeep` files**

Run:

```bash
New-Item -ItemType File -Path apps/.gitkeep -Force
New-Item -ItemType File -Path packages/.gitkeep -Force
```

On bash:

```bash
touch apps/.gitkeep packages/.gitkeep
```

- [ ] **Step 4: Update root docs**

In `README.md`, add:

```markdown
- `apps/` - future web and mobile applications.
- `packages/` - future shared contracts, SDKs, and app packages.
```

In `docs/architecture/WORK_AREAS.md`, add a section:

```markdown
## Monorepo App And Package Areas

- `apps/` is for web/mobile applications.
- `packages/` is for shared app/cloud contracts, SDKs, and reusable app code.
- Device runtime code must not depend on `apps/`.
- Shared contracts should flow through `packages/contracts/` when that package exists.
```

- [ ] **Step 5: Commit Phase 6**

Run:

```bash
git add apps packages README.md docs/README.md docs/architecture/WORK_AREAS.md .gitignore
git commit -m "docs: add monorepo app and package roots"
```

---

## Phase 7: Final Validation And Hardware Prep

**Files:**

- No planned source files.
- Update PR body and docs only if validation changes the expected commands.

### Task 7.1: Full Local Validation

- [ ] **Step 1: Run full Rust validation**

Run:

```bash
cargo test --manifest-path yoyopod_rs/Cargo.toml --workspace --locked
cargo clippy --manifest-path yoyopod_rs/Cargo.toml --workspace --all-targets --locked -- -D warnings
cargo fmt --manifest-path yoyopod_rs/Cargo.toml --all --check
```

Expected:

- all pass

- [ ] **Step 2: Run targeted Python packaging/config validation**

Run:

```bash
uv run pytest -q tests/cli/test_yoyopod_cli_build.py
uv run pytest -q tests/cli/test_slot_contract.py
uv run pytest -q tests/scripts/test_build_release.py
uv run pytest -q tests/deploy/test_ci_workflows.py
uv run pytest -q tests/config/test_config_models.py -k voice
```

Expected:

- all pass

- [ ] **Step 3: Build local artifacts**

Run:

```bash
uv run yoyopod build rust-runtime
uv run yoyopod build voice-worker
```

Expected:

- `yoyopod_rs/runtime/build/yoyopod-runtime` exists
- `yoyopod_rs/speech/build/yoyopod-speech-host` exists after Phase 5 rename

- [ ] **Step 4: Check generated files are ignored**

Run:

```bash
git status --short --branch
git clean -ndX
git ls-files -ci --exclude-standard
git diff --check
```

Expected:

- only intentional source/doc changes are visible
- ignored build artifacts appear only in `git clean -ndX`
- `git ls-files -ci --exclude-standard` prints nothing
- `git diff --check` passes

### Task 7.2: Hardware Validation Prep

- [ ] **Step 1: Push branch and wait for CI artifacts**

Run:

```bash
BRANCH="$(git branch --show-current)"
COMMIT="$(git rev-parse HEAD)"
git push origin "$BRANCH"
printf 'branch=%s\ncommit=%s\n' "$BRANCH" "$COMMIT"
```

Expected:

- branch is pushed
- SHA is recorded for exact hardware validation

- [ ] **Step 2: Validate on Pi dev lane**

Run after CI artifact is available:

```bash
BRANCH="$(git branch --show-current)"
COMMIT="$(git rev-parse HEAD)"
yoyopod remote mode activate dev
yoyopod remote sync --branch "$BRANCH" --clean-native
yoyopod remote validate --branch "$BRANCH" --sha "$COMMIT"
```

Expected:

- dev lane runs Rust runtime
- worker binaries start from renamed paths
- UI, power, media, network, cloud, voip, and speech worker health checks report expected status for available hardware

- [ ] **Step 3: Update PR validation notes**

Generate the PR hardware-validation note from the current branch and commit:

```bash
BRANCH="$(git branch --show-current)"
COMMIT="$(git rev-parse HEAD)"
cat <<EOF
## Hardware Validation
- Branch: \`$BRANCH\`
- Commit: \`$COMMIT\`
- Artifact names: \`yoyopod-runtime\`, \`yoyopod-*-host\`
- Command: \`yoyopod remote validate --branch "$BRANCH" --sha "$COMMIT"\`
- Result: record the exact pass/fail lines printed by the validation command.
EOF
```

---

## Review Checklist

Before merging the full migration:

- [ ] `protocol` owns the only `WorkerEnvelope` definition.
- [ ] Domain crates do not depend on `runtime`.
- [ ] `worker` does not depend on `runtime`.
- [ ] `harness` is dev/test only.
- [ ] Deployed binary names still include `-host`.
- [ ] Source directories use domain names without `-host`.
- [ ] Slot packaging paths use new source directories.
- [ ] Config defaults use new source directories.
- [ ] `apps/` and `packages/` exist but contain no application code yet.
- [ ] No generated Rust target/build output is tracked.
- [ ] `git ls-files -ci --exclude-standard` prints nothing.
- [ ] Historical references are confined to historical docs.
