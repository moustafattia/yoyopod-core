# Rust-Only VoIP Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Rust the only supported VoIP runtime owner, including the liblinphone native shim, while Python remains only the app supervisor and event projector.

**Architecture:** Python starts the Rust VoIP worker, forwards app commands, and republishes Rust snapshots/events to the existing app bus. The Rust VoIP host owns calls, messages, voice notes, history, lifecycle, command policy, persistence, and the liblinphone binding. The current C liblinphone shim is replaced by a Rust `cdylib` exporting the same `yoyopod_liblinphone_*` ABI first, so the host/native boundary changes without also redesigning the worker protocol.

**Tech Stack:** Rust 2021 workspace under `yoyopod_rs/`, Rust `cdylib`, liblinphone C API, `bindgen`, `libloading`, NDJSON worker protocol, Python 3.12 supervisor, pytest, Cargo, Bazel, GitHub Actions ARM64 artifacts, Raspberry Pi Zero 2W.

---

## Current Base Reality

This implementation branch starts from `origin/main` after PR #414
(`0b1e796`), which replayed PR #412 and PR #413 onto main. Main now
contains the reviewed Rust-owned session lifecycle and message-store work.

Do not reapply the old #412/#413 stack on this branch. If the branch is
rebased later, verify that `yoyopod_rs/voip-host/src/message_store.rs`
still exists and that the VoIP host still owns session lifecycle snapshots
before continuing implementation work.

## File Structure

Create and modify these areas:

- Create: `yoyopod_rs/liblinphone-shim/`
  - Rust replacement for `yoyopod/backends/voip/shim_native/liblinphone_shim.c`.
  - Builds `libyoyopod_liblinphone_shim.so`.
  - Exports the current `yoyopod_liblinphone_*` ABI.
- Modify: `yoyopod_rs/Cargo.toml`
  - Add `liblinphone-shim` workspace member.
- Modify: `yoyopod_rs/voip-host/src/shim.rs`
  - Prefer Rust-built shim artifact paths.
  - Keep the existing dynamic ABI loader unless a later task explicitly removes it.
- Modify: `yoyopod_rs/voip-host/src/host.rs`
  - Add any missing command policy that still lives in Python.
- Modify: `yoyopod_rs/voip-host/src/worker.rs`
  - Add missing Rust-owned commands for call history and voice-note playback.
- Create: `yoyopod_rs/voip-host/src/history.rs`
  - Rust-owned call history persistence and seen/unseen state.
- Create: `yoyopod_rs/voip-host/src/playback.rs`
  - Rust-owned voice-note playback process control.
- Modify: `yoyopod_rs/voip-host/src/runtime_snapshot.rs`
  - Include final Rust-owned history/playback facts needed by Python/UI.
- Modify: `yoyopod/core/bootstrap/managers_boot.py`
  - Make Rust VoIP host the only production VoIP backend.
- Modify: `yoyopod/backends/voip/rust_host.py`
  - Keep as Python supervisor adapter only.
- Modify: `yoyopod/integrations/call/manager.py`
  - Collapse to snapshot-backed facade in Rust-only mode.
- Modify: `yoyopod/integrations/call/__init__.py`
  - Stop wiring old Python VoIP runtime ownership paths as active services.
- Delete or quarantine as non-runtime code:
  - `yoyopod/backends/voip/shim_native/`
  - `yoyopod/backends/voip/liblinphone.py`
  - `yoyopod/backends/voip/binding.py`
  - `yoyopod/backends/voip/supervisor_backed.py`
  - `yoyopod/integrations/call/sidecar_adapter.py`
  - `yoyopod/integrations/call/sidecar_main.py`
  - `yoyopod/integrations/call/sidecar_protocol.py`
  - `yoyopod/integrations/call/sidecar_supervisor.py`
- Modify: `.github/workflows/ci.yml`
  - Build and upload the Rust liblinphone shim artifact.
- Modify: `yoyopod_cli/build.py`
  - Replace `yoyopod build liblinphone` CMake shim build with Rust shim build.
- Modify: docs under `docs/operations/` and `docs/hardware/`
  - Document Rust-only VoIP and Rust shim artifact deployment.

## Non-Goals

- Do not migrate music/mpv into Rust.
- Do not migrate contacts into Rust.
- Do not migrate app bus, scheduler, or the whole Python supervisor into Rust.
- Do not rewrite the UI/LVGL shim in this PR.
- Do not keep Python sidecar or in-process liblinphone as runtime fallbacks.
- Do not build Rust binaries on the Pi; use CI artifacts for hardware validation.

## Task 0: Reconcile The Main Base With The Merged Rust VoIP Stack

**Files:**
- Inspect: `yoyopod_rs/voip-host/src/message_store.rs`
- Inspect: `yoyopod_rs/voip-host/src/lifecycle.rs`
- Inspect: `yoyopod_rs/voip-host/src/calls.rs`
- Inspect: `yoyopod/integrations/call/manager.py`

- [ ] **Step 1: Verify the implementation branch really starts from fresh main**

Run:

```powershell
git fetch origin main --prune
git switch -c codex/rust-only-voip-runtime origin/main
git log --oneline --decorate --max-count=5
```

Expected: the new branch starts at the current `origin/main`.

- [ ] **Step 2: Check whether main contains the #412/#413 behavior**

Run:

```powershell
Test-Path yoyopod_rs\voip-host\src\message_store.rs
rg "latest_voice_note_by_contact|mark_voice_notes_seen|MessageStore" -n yoyopod_rs\voip-host yoyopod\integrations\call
```

Expected: `message_store.rs` exists and the search finds Rust-owned voice-note summary/message-store behavior.

- [ ] **Step 3: If main lacks #412/#413, stop and fix the base**

Expected: this branch should not need any cherry-picks because #414 has
already landed the stack on main. If the check fails, stop and update the
base branch instead of continuing on a regressed VoIP host.

- [ ] **Step 4: Run the existing Rust/Python gates after base reconciliation**

Run:

```powershell
cargo test --workspace --locked
cargo test --workspace --locked --features whisplay-hardware
uv run python scripts/quality.py gate
uv run pytest -q
```

Expected: all pass. If any failure is caused by replay conflicts, fix it before continuing.

- [ ] **Step 5: Commit the reconciled base if cherry-picks were needed**

Run:

```powershell
git status --short
git commit -m "chore: replay merged rust voip ownership stack"
```

Expected: no reconciliation commit is needed when the branch starts from
`origin/main` at or after #414.

## Task 1: Add Rust Liblinphone Shim Crate Skeleton

**Files:**
- Modify: `yoyopod_rs/Cargo.toml`
- Create: `yoyopod_rs/liblinphone-shim/Cargo.toml`
- Create: `yoyopod_rs/liblinphone-shim/build.rs`
- Create: `yoyopod_rs/liblinphone-shim/src/lib.rs`
- Create: `yoyopod_rs/liblinphone-shim/src/error.rs`
- Create: `yoyopod_rs/liblinphone-shim/src/event.rs`
- Create: `yoyopod_rs/liblinphone-shim/src/state.rs`
- Test: `yoyopod_rs/liblinphone-shim/tests/abi.rs`

- [ ] **Step 1: Write failing ABI tests for exported version and error functions**

Create `yoyopod_rs/liblinphone-shim/tests/abi.rs`:

```rust
use std::ffi::CStr;

#[test]
fn rust_shim_exports_version_and_last_error_strings() {
    let version = unsafe { yoyopod_liblinphone_shim::yoyopod_liblinphone_version() };
    assert!(!version.is_null());
    let version = unsafe { CStr::from_ptr(version) }.to_string_lossy();
    assert!(version.contains("rust-liblinphone-shim"));

    let error = unsafe { yoyopod_liblinphone_shim::yoyopod_liblinphone_last_error() };
    assert!(!error.is_null());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-liblinphone-shim --test abi rust_shim_exports_version_and_last_error_strings
```

Expected: FAIL because the crate/package does not exist.

- [ ] **Step 3: Add crate to the workspace**

Modify `yoyopod_rs/Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "ui-host",
    "voip-host",
    "liblinphone-shim",
]
```

Create `yoyopod_rs/liblinphone-shim/Cargo.toml`:

```toml
[package]
name = "yoyopod-liblinphone-shim"
version = "0.1.0"
edition = "2021"

[lib]
name = "yoyopod_liblinphone_shim"
crate-type = ["cdylib", "rlib"]

[dependencies]
libc = "0.2"
once_cell = "1"
thiserror = "2"

[build-dependencies]
bindgen = "0.71"
pkg-config = "0.3"
```

Create `yoyopod_rs/liblinphone-shim/build.rs`:

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rustc-link-lib=linphone");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .allowlist_type("Linphone.*")
        .allowlist_function("linphone_.*")
        .allowlist_var("Linphone.*")
        .generate()
        .expect("generate liblinphone bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("write liblinphone bindings");
}
```

Create `yoyopod_rs/liblinphone-shim/wrapper.h`:

```c
#include <linphone/core.h>
```

- [ ] **Step 4: Add minimal exported ABI**

Create `yoyopod_rs/liblinphone-shim/src/lib.rs`:

```rust
mod error;
mod event;
mod state;

use std::os::raw::c_char;

pub use event::YoyopodLiblinphoneEvent;

static VERSION: &[u8] = b"rust-liblinphone-shim/0.1.0\0";

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_version() -> *const c_char {
    VERSION.as_ptr().cast()
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_last_error() -> *const c_char {
    error::last_error_ptr()
}
```

Create `yoyopod_rs/liblinphone-shim/src/error.rs`:

```rust
use std::cell::RefCell;
use std::ffi::CString;
use std::os::raw::c_char;

thread_local! {
    static LAST_ERROR: RefCell<CString> =
        RefCell::new(CString::new("no error").expect("static string"));
}

pub fn set_last_error(message: impl AsRef<str>) {
    let sanitized = message.as_ref().replace('\0', " ");
    LAST_ERROR.with(|slot| {
        *slot.borrow_mut() = CString::new(sanitized).expect("nul was sanitized");
    });
}

pub fn last_error_ptr() -> *const c_char {
    LAST_ERROR.with(|slot| slot.borrow().as_ptr())
}
```

Create `yoyopod_rs/liblinphone-shim/src/event.rs`:

```rust
use std::os::raw::c_char;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct YoyopodLiblinphoneEvent {
    pub event_type: i32,
    pub registration_state: i32,
    pub call_state: i32,
    pub message_kind: i32,
    pub message_direction: i32,
    pub message_delivery_state: i32,
    pub duration_ms: i32,
    pub unread: i32,
    pub message_id: [c_char; 128],
    pub peer_sip_address: [c_char; 256],
    pub sender_sip_address: [c_char; 256],
    pub recipient_sip_address: [c_char; 256],
    pub local_file_path: [c_char; 512],
    pub mime_type: [c_char; 128],
    pub text: [c_char; 1024],
    pub reason: [c_char; 256],
}

impl Default for YoyopodLiblinphoneEvent {
    fn default() -> Self {
        unsafe { std::mem::zeroed() }
    }
}
```

Create `yoyopod_rs/liblinphone-shim/src/state.rs`:

```rust
pub struct ShimState {
    pub initialized: bool,
}

impl ShimState {
    pub fn new() -> Self {
        Self { initialized: false }
    }
}
```

- [ ] **Step 5: Run the ABI test**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-liblinphone-shim --test abi rust_shim_exports_version_and_last_error_strings
```

Expected: PASS on machines with liblinphone headers installed. If local Windows lacks liblinphone headers, run this on the ARM CI runner and keep local verification to non-binding tests.

- [ ] **Step 6: Commit**

Run:

```powershell
git add yoyopod_rs/Cargo.toml yoyopod_rs/liblinphone-shim
git commit -m "feat(voip): add rust liblinphone shim crate"
```

## Task 2: Export The Existing Liblinphone Shim ABI From Rust

**Files:**
- Modify: `yoyopod_rs/liblinphone-shim/src/lib.rs`
- Modify: `yoyopod_rs/liblinphone-shim/src/state.rs`
- Modify: `yoyopod_rs/liblinphone-shim/src/event.rs`
- Test: `yoyopod_rs/liblinphone-shim/tests/abi.rs`

- [ ] **Step 1: Write failing tests for the full exported symbol surface**

Append to `yoyopod_rs/liblinphone-shim/tests/abi.rs`:

```rust
#[test]
fn rust_shim_exports_current_c_shim_abi_surface() {
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_init as unsafe extern "C" fn() -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_shutdown as unsafe extern "C" fn();
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_stop as unsafe extern "C" fn();
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_iterate as unsafe extern "C" fn();
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_poll_event
        as unsafe extern "C" fn(*mut yoyopod_liblinphone_shim::YoyopodLiblinphoneEvent) -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_make_call
        as unsafe extern "C" fn(*const std::os::raw::c_char) -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_answer_call as unsafe extern "C" fn() -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_reject_call as unsafe extern "C" fn() -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_hangup as unsafe extern "C" fn() -> i32;
    let _ = yoyopod_liblinphone_shim::yoyopod_liblinphone_set_muted as unsafe extern "C" fn(i32) -> i32;
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-liblinphone-shim --test abi rust_shim_exports_current_c_shim_abi_surface
```

Expected: FAIL because exported functions are missing.

- [ ] **Step 3: Add the exported function stubs backed by explicit errors**

Add to `yoyopod_rs/liblinphone-shim/src/lib.rs`:

```rust
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::sync::Mutex;

static STATE: Lazy<Mutex<state::ShimState>> = Lazy::new(|| Mutex::new(state::ShimState::new()));

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_init() -> c_int {
    let mut state = STATE.lock().expect("shim state lock");
    state.initialized = true;
    0
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_shutdown() {
    let mut state = STATE.lock().expect("shim state lock");
    *state = state::ShimState::new();
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_start(
    _sip_server: *const c_char,
    _sip_username: *const c_char,
    _sip_password: *const c_char,
    _sip_password_ha1: *const c_char,
    _sip_identity: *const c_char,
    _factory_config_path: *const c_char,
    _transport: *const c_char,
    _stun_server: *const c_char,
    _conference_factory_uri: *const c_char,
    _file_transfer_server_url: *const c_char,
    _lime_server_url: *const c_char,
    _auto_download_incoming_voice_recordings: i32,
    _playback_device_id: *const c_char,
    _ringer_device_id: *const c_char,
    _capture_device_id: *const c_char,
    _media_device_id: *const c_char,
    _echo_cancellation: i32,
    _mic_gain: i32,
    _output_volume: i32,
    _voice_note_store_dir: *const c_char,
) -> c_int {
    error::set_last_error("liblinphone start not wired yet");
    -1
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_stop() {}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_iterate() {}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_poll_event(_event_out: *mut event::YoyopodLiblinphoneEvent) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_make_call(_sip_address: *const c_char) -> c_int {
    error::set_last_error("make_call not wired yet");
    -1
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_answer_call() -> c_int {
    error::set_last_error("answer_call not wired yet");
    -1
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_reject_call() -> c_int {
    error::set_last_error("reject_call not wired yet");
    -1
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_hangup() -> c_int {
    error::set_last_error("hangup not wired yet");
    -1
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_set_muted(_muted: i32) -> c_int {
    error::set_last_error("set_muted not wired yet");
    -1
}
```

- [ ] **Step 4: Add string helpers and message/voice-note ABI stubs**

Append to `yoyopod_rs/liblinphone-shim/src/lib.rs`:

```rust
fn copy_str_to_c_buffer(value: &str, out: *mut c_char, out_size: u32) -> bool {
    if out.is_null() || out_size == 0 {
        return false;
    }
    let bytes = value.as_bytes();
    let writable = out_size.saturating_sub(1) as usize;
    let count = bytes.len().min(writable);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), out.cast::<u8>(), count);
        *out.add(count) = 0;
    }
    true
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_send_text_message(
    _sip_address: *const c_char,
    _text: *const c_char,
    message_id_out: *mut c_char,
    message_id_out_size: u32,
) -> c_int {
    copy_str_to_c_buffer("", message_id_out, message_id_out_size);
    error::set_last_error("send_text_message not wired yet");
    -1
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_start_voice_recording(_file_path: *const c_char) -> c_int {
    error::set_last_error("start_voice_recording not wired yet");
    -1
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_stop_voice_recording(duration_ms_out: *mut i32) -> c_int {
    if !duration_ms_out.is_null() {
        unsafe { *duration_ms_out = 0 };
    }
    error::set_last_error("stop_voice_recording not wired yet");
    -1
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_cancel_voice_recording() -> c_int {
    error::set_last_error("cancel_voice_recording not wired yet");
    -1
}

#[no_mangle]
pub extern "C" fn yoyopod_liblinphone_send_voice_note(
    _sip_address: *const c_char,
    _file_path: *const c_char,
    _duration_ms: i32,
    _mime_type: *const c_char,
    message_id_out: *mut c_char,
    message_id_out_size: u32,
) -> c_int {
    copy_str_to_c_buffer("", message_id_out, message_id_out_size);
    error::set_last_error("send_voice_note not wired yet");
    -1
}
```

- [ ] **Step 5: Run the ABI tests**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-liblinphone-shim --test abi
```

Expected: PASS. The functions exist and return explicit errors until Task 3 wires liblinphone.

- [ ] **Step 6: Commit**

Run:

```powershell
git add yoyopod_rs/liblinphone-shim
git commit -m "feat(voip): export liblinphone shim abi from rust"
```

## Task 3: Port Liblinphone Runtime Binding From C To Rust

**Files:**
- Modify: `yoyopod_rs/liblinphone-shim/src/state.rs`
- Modify: `yoyopod_rs/liblinphone-shim/src/event.rs`
- Modify: `yoyopod_rs/liblinphone-shim/src/lib.rs`
- Reference: `yoyopod/backends/voip/shim_native/liblinphone_shim.c`
- Test: `yoyopod_rs/liblinphone-shim/tests/event_queue.rs`

- [ ] **Step 1: Write event queue tests independent of liblinphone**

Create `yoyopod_rs/liblinphone-shim/tests/event_queue.rs`:

```rust
use yoyopod_liblinphone_shim::event::{EventQueue, YoyopodLiblinphoneEvent};

#[test]
fn event_queue_polls_fifo_events() {
    let queue = EventQueue::default();
    let mut first = YoyopodLiblinphoneEvent::default();
    first.event_type = 1;
    let mut second = YoyopodLiblinphoneEvent::default();
    second.event_type = 2;

    queue.push(first);
    queue.push(second);

    assert_eq!(queue.pop().expect("first").event_type, 1);
    assert_eq!(queue.pop().expect("second").event_type, 2);
    assert!(queue.pop().is_none());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-liblinphone-shim --test event_queue
```

Expected: FAIL because `EventQueue` does not exist.

- [ ] **Step 3: Implement event queue**

Add to `yoyopod_rs/liblinphone-shim/src/event.rs`:

```rust
use std::collections::VecDeque;
use std::sync::Mutex;

#[derive(Default)]
pub struct EventQueue {
    inner: Mutex<VecDeque<YoyopodLiblinphoneEvent>>,
}

impl EventQueue {
    pub fn push(&self, event: YoyopodLiblinphoneEvent) {
        self.inner.expect("event queue lock").push_back(event);
    }

    pub fn pop(&self) -> Option<YoyopodLiblinphoneEvent> {
        self.inner.expect("event queue lock").pop_front()
    }
}
```

If `Mutex::expect` is not available in the target Rust toolchain, use:

```rust
self.inner.lock().expect("event queue lock")
```

- [ ] **Step 4: Run the event queue test**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-liblinphone-shim --test event_queue
```

Expected: PASS.

- [ ] **Step 5: Replace stub state with liblinphone-owned state**

Modify `yoyopod_rs/liblinphone-shim/src/state.rs` so it stores the liblinphone core, current call, active recorder path, event queue, and owned CString buffers needed by callbacks:

```rust
use crate::event::EventQueue;

pub struct ShimState {
    pub initialized: bool,
    pub running: bool,
    pub events: EventQueue,
}

impl ShimState {
    pub fn new() -> Self {
        Self {
            initialized: false,
            running: false,
            events: EventQueue::default(),
        }
    }
}
```

Then port the existing C state fields from `liblinphone_shim.c` one field at a time into Rust. Preserve these behaviors:

- one process-global core
- one current active call
- one active voice-note recorder
- event queue drained by `yoyopod_liblinphone_poll_event`
- `last_error` set on every failed public ABI call

- [ ] **Step 6: Wire `init`, `start`, `stop`, `shutdown`, and `iterate` to liblinphone**

Port the C behavior from:

- `yoyopod_liblinphone_init`
- `yoyopod_liblinphone_start`
- `yoyopod_liblinphone_stop`
- `yoyopod_liblinphone_shutdown`
- `yoyopod_liblinphone_iterate`

Keep the exported Rust function signatures unchanged. Use the generated bindings from `bindgen` for `linphone_core_new`, proxy/auth setup, audio device setup, callbacks, and iterate.

- [ ] **Step 7: Wire registration and call callbacks into Rust event queue**

Port the C callback behavior that produces native event types:

- `1`: registration changed
- `2`: call state changed
- `3`: incoming call
- `4`: backend stopped

Add unit tests for pure enum mapping in `yoyopod_rs/liblinphone-shim/tests/event_queue.rs`:

```rust
#[test]
fn event_type_values_match_existing_host_loader_contract() {
    assert_eq!(yoyopod_liblinphone_shim::event::EVENT_REGISTRATION_CHANGED, 1);
    assert_eq!(yoyopod_liblinphone_shim::event::EVENT_CALL_STATE_CHANGED, 2);
    assert_eq!(yoyopod_liblinphone_shim::event::EVENT_INCOMING_CALL, 3);
    assert_eq!(yoyopod_liblinphone_shim::event::EVENT_BACKEND_STOPPED, 4);
}
```

- [ ] **Step 8: Wire call commands**

Port the behavior for:

- `yoyopod_liblinphone_make_call`
- `yoyopod_liblinphone_answer_call`
- `yoyopod_liblinphone_reject_call`
- `yoyopod_liblinphone_hangup`
- `yoyopod_liblinphone_set_muted`

Expected: all return `0` when liblinphone accepts the operation and `-1` with `last_error` otherwise.

- [ ] **Step 9: Wire message and voice-note commands**

Port the behavior for:

- `yoyopod_liblinphone_send_text_message`
- `yoyopod_liblinphone_start_voice_recording`
- `yoyopod_liblinphone_stop_voice_recording`
- `yoyopod_liblinphone_cancel_voice_recording`
- `yoyopod_liblinphone_send_voice_note`

Preserve the event field sizes from `liblinphone_shim.h` exactly because `yoyopod_rs/voip-host/src/shim.rs` depends on that ABI layout.

- [ ] **Step 10: Run Rust shim tests**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-liblinphone-shim --locked
```

Expected: PASS on a system with liblinphone headers/libs.

- [ ] **Step 11: Commit**

Run:

```powershell
git add yoyopod_rs/liblinphone-shim
git commit -m "feat(voip): port liblinphone shim runtime to rust"
```

## Task 4: Build And Load The Rust Shim By Default

**Files:**
- Modify: `yoyopod_rs/voip-host/src/shim.rs`
- Modify: `yoyopod_rs/voip-host/tests/shim.rs`
- Modify: `yoyopod_cli/build.py`
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Write failing Rust host shim path test**

Modify `yoyopod_rs/voip-host/tests/shim.rs` to assert the Rust shim candidate is first:

```rust
#[test]
fn default_shim_candidates_prefer_rust_liblinphone_shim_artifact() {
    let candidates = yoyopod_voip_host::shim::default_shim_candidates(std::path::Path::new("/repo"));
    assert_eq!(
        candidates[0],
        std::path::Path::new("/repo")
            .join("yoyopod_rs")
            .join("liblinphone-shim")
            .join("build")
            .join("libyoyopod_liblinphone_shim.so")
    );
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-voip-host --test shim default_shim_candidates_prefer_rust_liblinphone_shim_artifact
```

Expected: FAIL because the current default path prefers the C shim.

- [ ] **Step 3: Implement Rust shim candidate resolution**

Modify `yoyopod_rs/voip-host/src/shim.rs`:

```rust
pub fn default_shim_candidates(repo_root: &Path) -> Vec<PathBuf> {
    vec![
        repo_root
            .join("yoyopod_rs")
            .join("liblinphone-shim")
            .join("build")
            .join(shim_file_name()),
        repo_root
            .join("yoyopod")
            .join("backends")
            .join("voip")
            .join("shim_native")
            .join("build")
            .join(shim_file_name()),
    ]
}
```

Update `resolve_shim_path` to use this helper and prefer the Rust path.

- [ ] **Step 4: Replace Python build command for liblinphone shim**

Modify `yoyopod_cli/build.py` so `yoyopod build liblinphone` runs:

```powershell
cargo build --release -p yoyopod-liblinphone-shim --locked
```

Then copy:

```text
yoyopod_rs/target/release/libyoyopod_liblinphone_shim.so
```

to:

```text
yoyopod_rs/liblinphone-shim/build/libyoyopod_liblinphone_shim.so
```

- [ ] **Step 5: Update CI to build and upload the Rust shim artifact**

Modify `.github/workflows/ci.yml` `ui-rust` job:

```yaml
      - name: Install liblinphone build dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y liblinphone-dev clang libclang-dev pkg-config

      - name: Build Rust Liblinphone shim
        working-directory: yoyopod_rs
        run: |
          set -euo pipefail
          cargo build --release -p yoyopod-liblinphone-shim --locked
          mkdir -p liblinphone-shim/build
          cp target/release/libyoyopod_liblinphone_shim.so liblinphone-shim/build/libyoyopod_liblinphone_shim.so

      - name: Upload Rust Liblinphone shim ARM64 artifact
        uses: actions/upload-artifact@v4
        with:
          name: yoyopod-liblinphone-shim-${{ github.sha }}
          path: yoyopod_rs/liblinphone-shim/build/libyoyopod_liblinphone_shim.so
          if-no-files-found: error
```

- [ ] **Step 6: Run tests**

Run:

```powershell
cd yoyopod_rs
cargo test --workspace --locked
```

Expected: PASS.

- [ ] **Step 7: Commit**

Run:

```powershell
git add yoyopod_rs/voip-host/src/shim.rs yoyopod_rs/voip-host/tests/shim.rs yoyopod_cli/build.py .github/workflows/ci.yml
git commit -m "feat(voip): load rust liblinphone shim by default"
```

## Task 5: Move Remaining VoIP Domain State Into Rust

**Files:**
- Create: `yoyopod_rs/voip-host/src/history.rs`
- Create: `yoyopod_rs/voip-host/src/playback.rs`
- Modify: `yoyopod_rs/voip-host/src/lib.rs`
- Modify: `yoyopod_rs/voip-host/src/host.rs`
- Modify: `yoyopod_rs/voip-host/src/worker.rs`
- Modify: `yoyopod_rs/voip-host/src/runtime_snapshot.rs`
- Test: `yoyopod_rs/voip-host/tests/history.rs`
- Test: `yoyopod_rs/voip-host/tests/playback.rs`
- Test: `yoyopod_rs/voip-host/tests/worker.rs`

- [ ] **Step 1: Write failing call-history Rust tests**

Create `yoyopod_rs/voip-host/tests/history.rs`:

```rust
use yoyopod_voip_host::history::{CallHistoryEntry, CallHistoryStore};

#[test]
fn call_history_records_terminal_call_and_marks_seen() {
    let mut store = CallHistoryStore::memory(20);
    store.record(CallHistoryEntry {
        session_id: "call-1".to_string(),
        peer_sip_address: "sip:mom@example.com".to_string(),
        direction: "incoming".to_string(),
        outcome: "missed".to_string(),
        duration_seconds: 0,
        seen: false,
    });

    assert_eq!(store.unseen_count(), 1);
    store.mark_seen("sip:mom@example.com");
    assert_eq!(store.unseen_count(), 0);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-voip-host --test history
```

Expected: FAIL because `history` module does not exist.

- [ ] **Step 3: Implement Rust call history module**

Create `yoyopod_rs/voip-host/src/history.rs`:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallHistoryEntry {
    pub session_id: String,
    pub peer_sip_address: String,
    pub direction: String,
    pub outcome: String,
    pub duration_seconds: u64,
    pub seen: bool,
}

#[derive(Debug, Clone)]
pub struct CallHistoryStore {
    max_entries: usize,
    entries: Vec<CallHistoryEntry>,
}

impl CallHistoryStore {
    pub fn memory(max_entries: usize) -> Self {
        Self {
            max_entries: max_entries.max(1),
            entries: Vec::new(),
        }
    }

    pub fn record(&mut self, entry: CallHistoryEntry) {
        self.entries.insert(0, entry);
        self.entries.truncate(self.max_entries);
    }

    pub fn mark_seen(&mut self, peer_sip_address: &str) {
        for entry in &mut self.entries {
            if entry.peer_sip_address == peer_sip_address {
                entry.seen = true;
            }
        }
    }

    pub fn unseen_count(&self) -> usize {
        self.entries.iter().filter(|entry| !entry.seen).count()
    }
}
```

Modify `yoyopod_rs/voip-host/src/lib.rs`:

```rust
pub mod history;
```

- [ ] **Step 4: Add playback process abstraction tests**

Create `yoyopod_rs/voip-host/tests/playback.rs`:

```rust
use yoyopod_voip_host::playback::VoiceNotePlayback;

#[test]
fn playback_command_uses_aplay_quiet_mode() {
    assert_eq!(
        VoiceNotePlayback::command_for("/tmp/note.wav"),
        vec!["aplay".to_string(), "-q".to_string(), "/tmp/note.wav".to_string()]
    );
}
```

- [ ] **Step 5: Implement Rust playback module**

Create `yoyopod_rs/voip-host/src/playback.rs`:

```rust
pub struct VoiceNotePlayback;

impl VoiceNotePlayback {
    pub fn command_for(file_path: &str) -> Vec<String> {
        vec!["aplay".to_string(), "-q".to_string(), file_path.to_string()]
    }
}
```

Modify `yoyopod_rs/voip-host/src/lib.rs`:

```rust
pub mod playback;
```

- [ ] **Step 6: Wire history and playback into `VoipHost`**

Modify `yoyopod_rs/voip-host/src/host.rs`:

- Add `call_history: CallHistoryStore`.
- Add `play_voice_note(file_path: &str) -> Result<(), String>`.
- Add `mark_history_seen(peer_sip_address: &str)`.
- Record terminal call sessions when `CallSession` becomes inactive.
- Include history counts and playback state in `session_snapshot_payload`.

- [ ] **Step 7: Add worker commands**

Modify `yoyopod_rs/voip-host/src/worker.rs` to handle:

- `voip.play_voice_note`
- `voip.stop_voice_note_playback`
- `voip.mark_call_history_seen`

Expected command payloads:

```json
{"file_path":"/tmp/note.wav"}
{"sip_address":"sip:mom@example.com"}
```

- [ ] **Step 8: Run Rust host tests**

Run:

```powershell
cd yoyopod_rs
cargo test -p yoyopod-voip-host --test history --test playback --test host --test worker
```

Expected: PASS.

- [ ] **Step 9: Commit**

Run:

```powershell
git add yoyopod_rs/voip-host/src yoyopod_rs/voip-host/tests
git commit -m "feat(voip): move history and playback ownership into rust"
```

## Task 6: Make Rust VoIP Host The Only Runtime Path In Python

**Files:**
- Modify: `yoyopod/core/bootstrap/managers_boot.py`
- Modify: `yoyopod/backends/voip/__init__.py`
- Modify: `yoyopod/backends/voip/rust_host.py`
- Delete: `yoyopod/backends/voip/liblinphone.py`
- Delete: `yoyopod/backends/voip/binding.py`
- Delete: `yoyopod/backends/voip/supervisor_backed.py`
- Delete: `yoyopod/integrations/call/sidecar_adapter.py`
- Delete: `yoyopod/integrations/call/sidecar_main.py`
- Delete: `yoyopod/integrations/call/sidecar_protocol.py`
- Delete: `yoyopod/integrations/call/sidecar_supervisor.py`
- Test: `tests/core/test_bootstrap.py`
- Test: `tests/backends/test_voip_backend.py`

- [ ] **Step 1: Write failing bootstrap test for Rust-only VoIP**

Modify `tests/core/test_bootstrap.py` with a test that clears old env flags and asserts `RustHostBackend` is selected.

```python
def test_voip_boot_uses_rust_host_without_fallback_flags(monkeypatch, app_factory):
    monkeypatch.delenv("YOYOPOD_RUST_VOIP_HOST", raising=False)
    monkeypatch.delenv("YOYOPOD_VOIP_SIDECAR", raising=False)
    app = app_factory()

    assert app.managers_boot.init_managers()

    from yoyopod.backends.voip.rust_host import RustHostBackend

    assert isinstance(app.voip_manager.backend, RustHostBackend)
```

Use the existing app/bootstrap test helpers in this file instead of introducing a new fixture if `app_factory` is not the local fixture name.

- [ ] **Step 2: Run test to verify it fails**

Run:

```powershell
uv run pytest -q tests/core/test_bootstrap.py::test_voip_boot_uses_rust_host_without_fallback_flags
```

Expected: FAIL because current boot still chooses old sidecar by default unless `YOYOPOD_RUST_VOIP_HOST=1`.

- [ ] **Step 3: Remove backend selection fallback**

Modify `yoyopod/core/bootstrap/managers_boot.py`:

- Delete `_voip_sidecar_enabled`.
- Delete `_rust_voip_host_enabled`.
- Always construct `RustHostBackend`.
- Always set `background_iterate_enabled = False`.
- Keep `YOYOPOD_RUST_VOIP_HOST_WORKER` as the worker path override.

Expected runtime path:

```python
from yoyopod.backends.voip.rust_host import RustHostBackend

sidecar_backed_backend = RustHostBackend(
    voip_config,
    worker_supervisor=self.app.worker_supervisor,
    worker_path=_rust_voip_host_worker_path(),
)
background_iterate_enabled = False
```

- [ ] **Step 4: Delete old runtime backend exports**

Modify `yoyopod/backends/voip/__init__.py` so it only exports:

- `MockVoIPBackend`
- `RustHostBackend`
- protocol types still used by tests/app code

Delete imports for `LiblinphoneBackend` and `SupervisorBackedBackend`.

- [ ] **Step 5: Delete old sidecar and in-process runtime files**

Run:

```powershell
git rm yoyopod/backends/voip/liblinphone.py
git rm yoyopod/backends/voip/binding.py
git rm yoyopod/backends/voip/supervisor_backed.py
git rm yoyopod/integrations/call/sidecar_adapter.py
git rm yoyopod/integrations/call/sidecar_main.py
git rm yoyopod/integrations/call/sidecar_protocol.py
git rm yoyopod/integrations/call/sidecar_supervisor.py
```

- [ ] **Step 6: Update tests importing deleted runtime paths**

Run:

```powershell
rg "LiblinphoneBackend|SupervisorBackedBackend|sidecar_adapter|sidecar_protocol|sidecar_supervisor|sidecar_main|YOYOPOD_VOIP_SIDECAR" -n tests yoyopod
```

For each result:

- Update production code to use `RustHostBackend`.
- Update tests to assert Rust-only behavior.
- Delete tests whose only purpose was fallback compatibility.

- [ ] **Step 7: Run Python bootstrap/backend tests**

Run:

```powershell
uv run pytest -q tests/core/test_bootstrap.py tests/backends/test_rust_host_voip.py tests/backends/test_voip_backend.py
```

Expected: PASS.

- [ ] **Step 8: Commit**

Run:

```powershell
git add yoyopod tests
git commit -m "refactor(voip): make rust host the only runtime backend"
```

## Task 7: Collapse Python VoIPManager Into Supervisor And Snapshot Projector

**Files:**
- Modify: `yoyopod/integrations/call/manager.py`
- Modify: `yoyopod/integrations/call/__init__.py`
- Modify: `yoyopod/integrations/call/handlers.py`
- Test: `tests/backends/test_voip_backend.py`
- Test: `tests/integrations/test_voip_services.py`

- [ ] **Step 1: Write failing tests proving Python services are not source of truth**

Add tests to `tests/backends/test_voip_backend.py`:

```python
def test_rust_only_voip_manager_getters_read_latest_rust_snapshot(tmp_path):
    backend = SnapshotOwnedMockVoIPBackend()
    manager = VoIPManager(build_config(tmp_path), backend=backend)

    snapshot = build_runtime_snapshot(
        call_state=CallState.CONNECTED,
        active_call_id="call-1",
        active_call_peer="sip:mom@example.com",
        muted=True,
    )
    backend.runtime_snapshot = snapshot
    backend.emit(VoIPRuntimeSnapshotChanged(snapshot=snapshot))

    assert manager.get_runtime_snapshot() == snapshot
    assert manager.call_state == CallState.CONNECTED
    assert manager.current_call_id == "call-1"
    assert manager.caller_address == "sip:mom@example.com"
    assert manager.is_muted is True
```

Add a second test:

```python
def test_rust_only_voip_manager_does_not_write_python_message_store(tmp_path):
    backend = SnapshotOwnedMockVoIPBackend()
    manager = VoIPManager(build_config(tmp_path), backend=backend)
    assert manager.start()

    backend.emit(MessageReceived(message=voice_note_record("note-1")))

    assert manager._message_store.get("note-1") is None
```

- [ ] **Step 2: Run tests to verify current duplicate ownership fails**

Run:

```powershell
uv run pytest -q tests/backends/test_voip_backend.py::test_rust_only_voip_manager_getters_read_latest_rust_snapshot tests/backends/test_voip_backend.py::test_rust_only_voip_manager_does_not_write_python_message_store
```

Expected: At least one fails until the manager is collapsed.

- [ ] **Step 3: Collapse Rust-only manager state paths**

Modify `yoyopod/integrations/call/manager.py`:

- Keep public methods and callbacks.
- Remove Python-side live state mutation that duplicates Rust snapshots.
- Command methods call `self.backend` and return the backend result.
- Getter methods read `self._runtime_snapshot`.
- Event handling maps Rust events to callbacks/app events only.
- Python `MessagingService`, `VoiceNoteService`, `CallRuntime`, and `VoIPMessageStore` are not used as source of truth.

Keep these methods as adapter/projection methods:

- `start`
- `stop`
- `make_call`
- `answer_call`
- `hangup`
- `reject_call`
- `mute`
- `unmute`
- `toggle_mute`
- `send_text_message`
- `start_voice_note_recording`
- `stop_voice_note_recording`
- `cancel_voice_note_recording`
- `send_active_voice_note`
- `latest_voice_note_for_contact`
- `unread_voice_note_count`
- `latest_voice_note_summary`
- `mark_voice_notes_seen`
- `get_runtime_snapshot`
- callback registration methods

- [ ] **Step 4: Move command preconditions to Rust**

Remove Python-only preconditions such as:

- "Cannot start voice-note recording during active call"
- Python mute state toggles based on stale local state
- Python voice-note send state checks

Add equivalent Rust tests in `yoyopod_rs/voip-host/tests/host.rs`.

- [ ] **Step 5: Run focused tests**

Run:

```powershell
uv run pytest -q tests/backends/test_voip_backend.py tests/integrations/test_voip_services.py
cd yoyopod_rs
cargo test -p yoyopod-voip-host --test host
```

Expected: PASS.

- [ ] **Step 6: Commit**

Run:

```powershell
git add yoyopod/integrations/call tests yoyopod_rs/voip-host
git commit -m "refactor(voip): collapse python manager to rust snapshot projector"
```

## Task 8: Remove The C Liblinphone Shim Runtime Source

**Files:**
- Delete: `yoyopod/backends/voip/shim_native/CMakeLists.txt`
- Delete: `yoyopod/backends/voip/shim_native/liblinphone_shim.c`
- Delete: `yoyopod/backends/voip/shim_native/liblinphone_shim.h`
- Modify: `yoyopod_cli/build.py`
- Modify: docs referencing C shim build paths

- [ ] **Step 1: Search all C shim references**

Run:

```powershell
rg "shim_native|liblinphone_shim.c|liblinphone_shim.h|yoyopod_liblinphone_shim|build liblinphone" -n yoyopod yoyopod_cli docs tests .github
```

Expected: references exist in build/docs/tests.

- [ ] **Step 2: Delete C shim source**

Run:

```powershell
git rm -r yoyopod/backends/voip/shim_native
```

- [ ] **Step 3: Update build/docs references to Rust shim**

Replace references to:

```text
yoyopod/backends/voip/shim_native/build/libyoyopod_liblinphone_shim.so
```

with:

```text
yoyopod_rs/liblinphone-shim/build/libyoyopod_liblinphone_shim.so
```

- [ ] **Step 4: Run reference search again**

Run:

```powershell
rg "shim_native|liblinphone_shim.c|liblinphone_shim.h" -n yoyopod yoyopod_cli docs tests .github
```

Expected: no runtime/build references remain. Historical docs under `docs/history/` may remain if clearly historical.

- [ ] **Step 5: Commit**

Run:

```powershell
git add yoyopod yoyopod_cli docs tests .github
git commit -m "refactor(voip): remove c liblinphone shim runtime source"
```

## Task 9: CI, Artifact, And Hardware Deploy Contract

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `docs/hardware/DEPLOYED_PI_DEPENDENCIES.md`
- Modify: `docs/operations/PI_DEV_WORKFLOW.md`
- Modify: `skills/yoyopod-rust-artifact/SKILL.md`
- Modify: `yoyopod_cli/remote_validate.py`

- [ ] **Step 1: Update CI artifact names**

Ensure CI uploads:

- `yoyopod-voip-host-${{ github.sha }}`
- `yoyopod-liblinphone-shim-${{ github.sha }}`

- [ ] **Step 2: Update remote validation checks**

Modify `yoyopod_cli/remote_validate.py` to require both files before Rust VoIP validation:

```text
yoyopod_rs/voip-host/build/yoyopod-voip-host
yoyopod_rs/liblinphone-shim/build/libyoyopod_liblinphone_shim.so
```

The error message must say to download GitHub Actions artifacts for the exact commit and not build Rust on the Pi.

- [ ] **Step 3: Update deployment docs**

Document the runtime path:

```text
Python supervisor -> Rust VoIP host -> Rust liblinphone shim -> liblinphone
```

State explicitly:

- Python sidecar VoIP is removed.
- In-process Python liblinphone backend is removed.
- C liblinphone shim is removed.
- CI artifact deploy is required for Rust binaries/libraries.

- [ ] **Step 4: Commit**

Run:

```powershell
git add .github/workflows/ci.yml docs skills yoyopod_cli/remote_validate.py
git commit -m "docs(voip): document rust-only voip artifact contract"
```

## Task 10: Full Validation

**Files:**
- No planned source edits.

- [ ] **Step 1: Format Rust**

Run:

```powershell
cd yoyopod_rs
cargo fmt --all --check
```

Expected: PASS.

- [ ] **Step 2: Run Rust tests**

Run:

```powershell
cd yoyopod_rs
cargo test --workspace --locked
cargo test --workspace --locked --features whisplay-hardware
```

Expected: PASS.

- [ ] **Step 3: Run Bazel tests where available**

Run:

```powershell
bazel test //yoyopod_rs/ui-host/... //yoyopod_rs/voip-host/...
```

Expected: PASS on environments with Bazel/Bazelisk installed. If Bazel is not installed locally, report that local Bazel validation was unavailable and rely on CI.

- [ ] **Step 4: Run Python quality gate and tests**

Run:

```powershell
uv run python scripts/quality.py gate
uv run pytest -q
```

Expected: PASS.

- [ ] **Step 5: Push branch and wait for CI**

Run:

```powershell
git push -u origin codex/rust-only-voip-runtime
gh pr create --draft --title "Move VoIP fully to Rust runtime" --body-file docs/superpowers/plans/2026-04-29-rust-only-voip-runtime.md
gh pr checks --watch
```

Expected: CI passes, including ARM Rust host/shim artifact builds.

- [ ] **Step 6: Hardware validation**

After CI publishes artifacts for the exact commit:

```powershell
yoyopod remote mode activate dev
yoyopod remote sync --branch codex/rust-only-voip-runtime --clean-native
yoyopod pi validate smoke --with-rust-voip-host
```

Expected:

- Rust VoIP host starts.
- Rust liblinphone shim loads.
- SIP registration succeeds.
- Incoming call event reaches UI.
- Outgoing call can start and hang up.
- Text message command returns a client id.
- Voice-note record/send/playback path works.

## Plan Self-Review

- Spec coverage: The plan covers Rust-only VoIP runtime, Rust liblinphone shim, Python supervisor-only behavior, CI artifacts, and hardware validation.
- Vague-marker scan: No forbidden planning markers or intentionally vague implementation steps remain.
- Type consistency: Rust crate names use `yoyopod-liblinphone-shim` as package and `yoyopod_liblinphone_shim` as library/export namespace. Existing Rust host crate remains `yoyopod-voip-host`.
- Scope check: This is intentionally a large single PR. It does not include UI, music, contacts, network, power, or Python supervisor migration.
