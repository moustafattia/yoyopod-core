# Voice Trace And Dictionary Validator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a bounded JSONL voice trace and command dictionary validator so Pi voice failures can be diagnosed from structured local evidence.

**Architecture:** Add focused voice trace and validator modules under `yoyopod/integrations/voice/`, expose them through a new `yoyopod voice` CLI group, and opt the canonical app boot path into trace recording. Keep runtime tracing best-effort: trace write failures are logged and never affect voice, music, VoIP, or UI behavior.

**Tech Stack:** Python 3.12 dataclasses, JSONL, PyYAML, Typer, pytest, loguru.

---

## File Structure

- Create `yoyopod/integrations/voice/trace.py`: trace entry dataclasses, bounded JSONL store, text capping, corrupt-line tolerant reads.
- Create `yoyopod/integrations/voice/dictionary_validator.py`: strict dictionary validation with deterministic errors and warnings.
- Create `yoyopod_cli/voice.py`: `yoyopod voice trace last` and `yoyopod voice dictionary validate`.
- Modify `yoyopod/config/models/voice.py`: add `VoiceTraceConfig` and `VoiceConfig.trace`.
- Modify `yoyopod/config/models/__init__.py` and `yoyopod/config/__init__.py`: export `VoiceTraceConfig`.
- Modify `yoyopod/integrations/voice/models.py`: add trace settings to `VoiceSettings`.
- Modify `yoyopod/integrations/voice/settings.py`: resolve trace settings from `ConfigManager`.
- Modify `yoyopod/integrations/voice/runtime.py`: capture trace events around STT, route, outcome, speech, and music focus.
- Modify `yoyopod/core/bootstrap/screens_boot.py`: pass a trace-store factory in canonical app boot.
- Modify `yoyopod_cli/main.py`: register the voice CLI group.
- Modify `config/voice/assistant.yaml`: add authored prod-safe trace defaults.
- Modify `tests/config/test_config_models.py`: config defaults and env coverage.
- Create `tests/integrations/test_voice_trace.py`: trace store tests.
- Create `tests/integrations/test_voice_dictionary_validator.py`: validator tests.
- Create `tests/cli/test_yoyopod_cli_voice.py`: voice CLI tests.
- Modify `tests/integrations/test_voice_runtime.py`: runtime trace integration tests.
- Modify `tests/cli/test_yoyopod_cli_docgen.py`: command docs coverage.
- Regenerate `yoyopod_cli/COMMANDS.md` after CLI registration.

## Task 1: Trace Config And JSONL Store

**Files:**
- Create: `yoyopod/integrations/voice/trace.py`
- Create: `tests/integrations/test_voice_trace.py`
- Modify: `yoyopod/config/models/voice.py`
- Modify: `yoyopod/config/models/__init__.py`
- Modify: `yoyopod/config/__init__.py`
- Modify: `yoyopod/integrations/voice/models.py`
- Modify: `yoyopod/integrations/voice/settings.py`
- Modify: `config/voice/assistant.yaml`
- Modify: `tests/config/test_config_models.py`
- Modify: `tests/integrations/test_voice_runtime.py`

- [ ] **Step 1: Add failing config tests**

Append these tests to `tests/config/test_config_models.py`:

```python
def test_voice_trace_config_defaults_do_not_require_a_file(tmp_path, monkeypatch) -> None:
    """Missing voice config should still enable bounded voice trace defaults."""

    for key in [
        "YOYOPOD_VOICE_TRACE_ENABLED",
        "YOYOPOD_VOICE_TRACE_PATH",
        "YOYOPOD_VOICE_TRACE_MAX_TURNS",
        "YOYOPOD_VOICE_TRACE_INCLUDE_TRANSCRIPTS",
        "YOYOPOD_VOICE_TRACE_BODY_PREVIEW_CHARS",
    ]:
        monkeypatch.delenv(key, raising=False)

    settings = load_config_model_from_yaml(VoiceConfig, tmp_path / "voice.yaml")

    assert settings.trace.enabled is True
    assert settings.trace.path == "logs/voice/turns.jsonl"
    assert settings.trace.max_turns == 50
    assert settings.trace.include_transcripts is True
    assert settings.trace.body_preview_chars == 160


def test_voice_trace_config_env_overrides(tmp_path, monkeypatch) -> None:
    """Voice trace settings should be overridable through typed env fields."""

    monkeypatch.setenv("YOYOPOD_VOICE_TRACE_ENABLED", "false")
    monkeypatch.setenv("YOYOPOD_VOICE_TRACE_PATH", "/tmp/yoyopod/voice.jsonl")
    monkeypatch.setenv("YOYOPOD_VOICE_TRACE_MAX_TURNS", "200")
    monkeypatch.setenv("YOYOPOD_VOICE_TRACE_INCLUDE_TRANSCRIPTS", "false")
    monkeypatch.setenv("YOYOPOD_VOICE_TRACE_BODY_PREVIEW_CHARS", "64")

    settings = load_config_model_from_yaml(VoiceConfig, tmp_path / "voice.yaml")

    assert settings.trace.enabled is False
    assert settings.trace.path == "/tmp/yoyopod/voice.jsonl"
    assert settings.trace.max_turns == 200
    assert settings.trace.include_transcripts is False
    assert settings.trace.body_preview_chars == 64
```

Append this test near `test_voice_settings_resolver_includes_command_routing_config` in `tests/integrations/test_voice_runtime.py`:

```python
def test_voice_settings_resolver_includes_trace_config() -> None:
    config_manager = _FakeConfigManager([])
    voice_cfg = config_manager.get_voice_settings()
    voice_cfg.trace = SimpleNamespace(
        enabled=False,
        path="logs/voice/test-turns.jsonl",
        max_turns=123,
        include_transcripts=False,
        body_preview_chars=44,
    )

    settings = VoiceSettingsResolver(
        context=None,
        config_manager=config_manager,
    ).defaults()

    assert settings.voice_trace_enabled is False
    assert settings.voice_trace_path == "logs/voice/test-turns.jsonl"
    assert settings.voice_trace_max_turns == 123
    assert settings.voice_trace_include_transcripts is False
    assert settings.voice_trace_body_preview_chars == 44
```

- [ ] **Step 2: Run config tests to verify they fail**

Run:

```bash
uv run pytest tests/config/test_config_models.py::test_voice_trace_config_defaults_do_not_require_a_file tests/config/test_config_models.py::test_voice_trace_config_env_overrides tests/integrations/test_voice_runtime.py::test_voice_settings_resolver_includes_trace_config -q
```

Expected: FAIL because `VoiceConfig.trace`, `VoiceTraceConfig`, and `VoiceSettings.voice_trace_*` do not exist.

- [ ] **Step 3: Add failing trace store tests**

Create `tests/integrations/test_voice_trace.py`:

```python
"""Tests for bounded voice turn trace storage."""

from __future__ import annotations

import json
from pathlib import Path

from yoyopod.integrations.voice.trace import (
    DEFAULT_VOICE_TRACE_PATH,
    VoiceTraceEntry,
    VoiceTraceStore,
)


def test_trace_entry_caps_transcripts_and_body_preview() -> None:
    entry = VoiceTraceEntry(
        turn_id="turn-1",
        started_at="2026-04-28T10:00:00Z",
        completed_at="2026-04-28T10:00:01Z",
        source="ask_screen",
        mode="ask",
        route_kind="ask",
        outcome="answer",
        transcript_raw="one two three four",
        transcript_normalized="one two three four",
        assistant_body_preview="alpha beta gamma",
        include_transcripts=True,
        text_limit=9,
        body_preview_chars=7,
    )

    payload = entry.to_json_dict()

    assert payload["schema_version"] == 1
    assert payload["transcript_raw"] == "one tw..."
    assert payload["transcript_normalized"] == "one tw..."
    assert payload["assistant_body_preview"] == "alph..."


def test_trace_entry_can_omit_transcripts() -> None:
    entry = VoiceTraceEntry(
        turn_id="turn-1",
        started_at="2026-04-28T10:00:00Z",
        completed_at="2026-04-28T10:00:01Z",
        source="ask_screen",
        mode="ask",
        route_kind="command",
        outcome="command_started",
        transcript_raw="call mama",
        transcript_normalized="call mama",
        include_transcripts=False,
    )

    payload = entry.to_json_dict()

    assert "transcript_raw" not in payload
    assert "transcript_normalized" not in payload


def test_trace_store_appends_and_rotates_valid_jsonl(tmp_path: Path) -> None:
    trace_path = tmp_path / "voice" / "turns.jsonl"
    store = VoiceTraceStore(path=trace_path, max_turns=2)

    for index in range(3):
        store.append(
            VoiceTraceEntry(
                turn_id=f"turn-{index}",
                started_at=f"2026-04-28T10:00:0{index}Z",
                completed_at=f"2026-04-28T10:00:1{index}Z",
                source="ask_screen",
                mode="ask",
                route_kind="command",
                outcome="command_started",
            )
        )

    lines = trace_path.read_text(encoding="utf-8").splitlines()
    payloads = [json.loads(line) for line in lines]

    assert [payload["turn_id"] for payload in payloads] == ["turn-1", "turn-2"]
    assert store.read_recent(limit=10)[0]["turn_id"] == "turn-1"
    assert store.read_recent(limit=1)[0]["turn_id"] == "turn-2"


def test_trace_store_ignores_corrupt_lines_during_read_and_rotation(tmp_path: Path) -> None:
    trace_path = tmp_path / "turns.jsonl"
    trace_path.write_text(
        '{"turn_id":"old","schema_version":1}\nnot-json\n',
        encoding="utf-8",
    )
    store = VoiceTraceStore(path=trace_path, max_turns=2)

    store.append(
        VoiceTraceEntry(
            turn_id="new",
            started_at="2026-04-28T10:00:00Z",
            completed_at="2026-04-28T10:00:01Z",
            source="ask_screen",
            mode="ask",
            route_kind="error",
            outcome="stt_failed",
            error={"stage": "stt", "type": "RuntimeError", "message": "boom"},
        )
    )

    recent = store.read_recent(limit=10)

    assert [payload["turn_id"] for payload in recent] == ["old", "new"]
    assert all("not-json" not in line for line in trace_path.read_text(encoding="utf-8").splitlines())


def test_default_trace_path_is_voice_jsonl() -> None:
    assert DEFAULT_VOICE_TRACE_PATH == Path("logs/voice/turns.jsonl")
```

- [ ] **Step 4: Run trace store tests to verify they fail**

Run:

```bash
uv run pytest tests/integrations/test_voice_trace.py -q
```

Expected: FAIL because `yoyopod.integrations.voice.trace` does not exist.

- [ ] **Step 5: Implement trace config models and runtime settings**

In `yoyopod/config/models/voice.py`, add this dataclass after `VoiceAudioConfig`:

```python
@dataclass(slots=True)
class VoiceTraceConfig:
    """Bounded local voice turn trace policy."""

    enabled: bool = config_value(default=True, env="YOYOPOD_VOICE_TRACE_ENABLED")
    path: str = config_value(
        default="logs/voice/turns.jsonl",
        env="YOYOPOD_VOICE_TRACE_PATH",
    )
    max_turns: int = config_value(default=50, env="YOYOPOD_VOICE_TRACE_MAX_TURNS")
    include_transcripts: bool = config_value(
        default=True,
        env="YOYOPOD_VOICE_TRACE_INCLUDE_TRANSCRIPTS",
    )
    body_preview_chars: int = config_value(
        default=160,
        env="YOYOPOD_VOICE_TRACE_BODY_PREVIEW_CHARS",
    )
```

In `VoiceConfig`, add:

```python
    trace: VoiceTraceConfig = config_value(default_factory=VoiceTraceConfig)
```

In `yoyopod/config/models/__init__.py`, import and export `VoiceTraceConfig`:

```python
from yoyopod.config.models.voice import (
    VoiceAssistantConfig,
    VoiceAudioConfig,
    VoiceCommandRoutingConfig,
    VoiceConfig,
    VoiceTraceConfig,
    VoiceWorkerConfig,
)
```

Add `"VoiceTraceConfig"` to `__all__`.

In `yoyopod/config/__init__.py`, import and export `VoiceTraceConfig` the same way.

In `yoyopod/integrations/voice/models.py`, add these frozen `VoiceSettings` fields after `fallback_min_command_confidence`:

```python
    voice_trace_enabled: bool = True
    voice_trace_path: str = "logs/voice/turns.jsonl"
    voice_trace_max_turns: int = 50
    voice_trace_include_transcripts: bool = True
    voice_trace_body_preview_chars: int = 160
```

In `yoyopod/integrations/voice/settings.py`, capture `trace_cfg` in `defaults()`:

```python
        trace_cfg = None
        if self._config_manager is not None:
            voice_cfg = getattr(self._config_manager, "get_voice_settings", lambda: None)()
            if voice_cfg is not None:
                assistant_cfg = getattr(voice_cfg, "assistant", None)
                worker_cfg = getattr(voice_cfg, "worker", None)
                trace_cfg = getattr(voice_cfg, "trace", None)
```

Then add these fields in the returned `VoiceSettings(...)`:

```python
            voice_trace_enabled=getattr(
                trace_cfg,
                "enabled",
                defaults.voice_trace_enabled,
            ),
            voice_trace_path=getattr(
                trace_cfg,
                "path",
                defaults.voice_trace_path,
            ),
            voice_trace_max_turns=getattr(
                trace_cfg,
                "max_turns",
                defaults.voice_trace_max_turns,
            ),
            voice_trace_include_transcripts=getattr(
                trace_cfg,
                "include_transcripts",
                defaults.voice_trace_include_transcripts,
            ),
            voice_trace_body_preview_chars=getattr(
                trace_cfg,
                "body_preview_chars",
                defaults.voice_trace_body_preview_chars,
            ),
```

In `config/voice/assistant.yaml`, add this top-level section:

```yaml
trace:
  enabled: true
  path: "logs/voice/turns.jsonl"
  max_turns: 50
  include_transcripts: true
  body_preview_chars: 160
```

- [ ] **Step 6: Implement the trace store module**

Create `yoyopod/integrations/voice/trace.py`:

```python
"""Bounded JSONL trace for local voice turns."""

from __future__ import annotations

from dataclasses import dataclass, field
from datetime import UTC, datetime
import json
import os
from pathlib import Path
from typing import Any
from uuid import uuid4

from loguru import logger

DEFAULT_VOICE_TRACE_PATH = Path("logs/voice/turns.jsonl")
DEFAULT_TRANSCRIPT_LIMIT = 256


def utc_now_iso() -> str:
    """Return a compact UTC timestamp with a stable Z suffix."""

    return datetime.now(UTC).isoformat(timespec="milliseconds").replace("+00:00", "Z")


def new_turn_id() -> str:
    """Return a short stable-enough turn id for logs and trace correlation."""

    return uuid4().hex[:16]


def _cap_text(value: str | None, limit: int) -> str | None:
    if value is None:
        return None
    normalized = " ".join(str(value).strip().split())
    if not normalized:
        return ""
    safe_limit = max(1, int(limit))
    if len(normalized) <= safe_limit:
        return normalized
    if safe_limit <= 3:
        return normalized[:safe_limit]
    return normalized[: safe_limit - 3] + "..."


def _drop_none(payload: dict[str, Any]) -> dict[str, Any]:
    return {key: value for key, value in payload.items() if value is not None}


@dataclass(slots=True)
class VoiceTraceEntry:
    """One voice interaction trace entry."""

    turn_id: str
    started_at: str
    completed_at: str
    source: str
    mode: str
    route_kind: str
    outcome: str
    schema_version: int = 1
    transcript_raw: str | None = None
    transcript_normalized: str | None = None
    activation_prefix: str | None = None
    command_intent: str | None = None
    command_confidence: float | None = None
    route_name: str | None = None
    ask_fallback: bool | None = None
    assistant_status: str | None = None
    assistant_title: str | None = None
    assistant_body_preview: str | None = None
    should_speak: bool | None = None
    auto_return: bool | None = None
    timings_ms: dict[str, float] = field(default_factory=dict)
    audio_focus_before: dict[str, Any] = field(default_factory=dict)
    audio_focus_after: dict[str, Any] = field(default_factory=dict)
    music_before: dict[str, Any] = field(default_factory=dict)
    music_after: dict[str, Any] = field(default_factory=dict)
    error: dict[str, str] | None = None
    include_transcripts: bool = True
    text_limit: int = DEFAULT_TRANSCRIPT_LIMIT
    body_preview_chars: int = 160

    def to_json_dict(self) -> dict[str, Any]:
        """Return the persisted JSON object for this trace entry."""

        payload: dict[str, Any] = {
            "schema_version": self.schema_version,
            "turn_id": self.turn_id,
            "started_at": self.started_at,
            "completed_at": self.completed_at,
            "source": self.source,
            "mode": self.mode,
            "route_kind": self.route_kind,
            "outcome": self.outcome,
            "activation_prefix": self.activation_prefix,
            "command_intent": self.command_intent,
            "command_confidence": self.command_confidence,
            "route_name": self.route_name,
            "ask_fallback": self.ask_fallback,
            "assistant_status": self.assistant_status,
            "assistant_title": self.assistant_title,
            "assistant_body_preview": _cap_text(
                self.assistant_body_preview,
                self.body_preview_chars,
            ),
            "should_speak": self.should_speak,
            "auto_return": self.auto_return,
            "timings_ms": self.timings_ms or None,
            "audio_focus_before": self.audio_focus_before or None,
            "audio_focus_after": self.audio_focus_after or None,
            "music_before": self.music_before or None,
            "music_after": self.music_after or None,
            "error": self.error,
        }
        if self.include_transcripts:
            payload["transcript_raw"] = _cap_text(self.transcript_raw, self.text_limit)
            payload["transcript_normalized"] = _cap_text(
                self.transcript_normalized,
                self.text_limit,
            )
        return _drop_none(payload)


@dataclass(slots=True)
class VoiceTraceStore:
    """Append and rotate a bounded JSONL voice trace."""

    path: Path
    max_turns: int = 50

    @classmethod
    def from_settings(cls, settings: Any) -> VoiceTraceStore:
        return cls(
            path=Path(getattr(settings, "voice_trace_path", DEFAULT_VOICE_TRACE_PATH)),
            max_turns=max(1, int(getattr(settings, "voice_trace_max_turns", 50))),
        )

    def append(self, entry: VoiceTraceEntry) -> None:
        try:
            self.path.parent.mkdir(parents=True, exist_ok=True)
            existing = self._read_all_valid()
            existing.append(entry.to_json_dict())
            self._rewrite(existing[-max(1, self.max_turns) :])
        except Exception as exc:
            logger.debug("Voice trace append failed: {}", exc)

    def read_recent(self, *, limit: int) -> list[dict[str, Any]]:
        entries = self._read_all_valid()
        safe_limit = max(0, int(limit))
        if safe_limit == 0:
            return []
        return entries[-safe_limit:]

    def _read_all_valid(self) -> list[dict[str, Any]]:
        if not self.path.exists():
            return []
        entries: list[dict[str, Any]] = []
        try:
            for line in self.path.read_text(encoding="utf-8").splitlines():
                if not line.strip():
                    continue
                try:
                    payload = json.loads(line)
                except json.JSONDecodeError:
                    logger.debug("Ignoring corrupt voice trace line in {}", self.path)
                    continue
                if isinstance(payload, dict):
                    entries.append(payload)
        except OSError as exc:
            logger.debug("Voice trace read failed: {}", exc)
        return entries

    def _rewrite(self, entries: list[dict[str, Any]]) -> None:
        tmp_path = self.path.with_suffix(self.path.suffix + ".tmp")
        content = "".join(
            json.dumps(entry, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
            for entry in entries
        )
        tmp_path.write_text(content, encoding="utf-8")
        os.replace(tmp_path, self.path)
```

- [ ] **Step 7: Run Task 1 tests**

Run:

```bash
uv run pytest tests/config/test_config_models.py::test_voice_trace_config_defaults_do_not_require_a_file tests/config/test_config_models.py::test_voice_trace_config_env_overrides tests/integrations/test_voice_runtime.py::test_voice_settings_resolver_includes_trace_config tests/integrations/test_voice_trace.py -q
```

Expected: PASS.

- [ ] **Step 8: Run required gates and commit Task 1**

Run:

```bash
uv run python scripts/quality.py gate
uv run pytest -q
git add yoyopod/config/models/voice.py yoyopod/config/models/__init__.py yoyopod/config/__init__.py yoyopod/integrations/voice/models.py yoyopod/integrations/voice/settings.py yoyopod/integrations/voice/trace.py config/voice/assistant.yaml tests/config/test_config_models.py tests/integrations/test_voice_runtime.py tests/integrations/test_voice_trace.py
git commit -m "feat: add bounded voice trace store"
```

Expected: quality passes, full pytest passes, commit created.

## Task 2: Voice Trace CLI

**Files:**
- Create: `yoyopod_cli/voice.py`
- Create: `tests/cli/test_yoyopod_cli_voice.py`
- Modify: `yoyopod_cli/main.py`
- Modify: `tests/cli/test_yoyopod_cli_docgen.py`
- Modify: `yoyopod_cli/COMMANDS.md`

- [ ] **Step 1: Add failing CLI trace tests**

Create `tests/cli/test_yoyopod_cli_voice.py`:

```python
"""Tests for `yoyopod voice` diagnostics commands."""

from __future__ import annotations

import json
from pathlib import Path

from typer.testing import CliRunner

from yoyopod.integrations.voice.trace import VoiceTraceEntry, VoiceTraceStore
from yoyopod_cli.main import app


def test_voice_group_is_registered() -> None:
    runner = CliRunner()
    result = runner.invoke(app, ["voice", "--help"])

    assert result.exit_code == 0
    assert "trace" in result.output


def test_voice_trace_last_prints_recent_rows(tmp_path: Path) -> None:
    trace_path = tmp_path / "turns.jsonl"
    store = VoiceTraceStore(path=trace_path, max_turns=10)
    store.append(
        VoiceTraceEntry(
            turn_id="turn-1",
            started_at="2026-04-28T10:00:00Z",
            completed_at="2026-04-28T10:00:01Z",
            source="ask_screen",
            mode="ask",
            route_kind="command",
            outcome="command_started",
            transcript_normalized="call mama",
            command_intent="call_contact",
        )
    )
    store.append(
        VoiceTraceEntry(
            turn_id="turn-2",
            started_at="2026-04-28T10:00:02Z",
            completed_at="2026-04-28T10:00:03Z",
            source="hub_hold",
            mode="ask",
            route_kind="ask",
            outcome="answer",
            transcript_normalized="why is the sky blue",
            assistant_body_preview="Because sunlight scatters.",
        )
    )

    runner = CliRunner()
    result = runner.invoke(app, ["voice", "trace", "last", "--limit", "1", "--path", str(trace_path)])

    assert result.exit_code == 0, result.output
    assert "turn-2" in result.output
    assert "why is the sky blue" in result.output
    assert "turn-1" not in result.output


def test_voice_trace_last_tolerates_missing_file(tmp_path: Path) -> None:
    runner = CliRunner()
    result = runner.invoke(
        app,
        ["voice", "trace", "last", "--path", str(tmp_path / "missing.jsonl")],
    )

    assert result.exit_code == 0
    assert "No voice trace entries" in result.output


def test_voice_trace_last_ignores_corrupt_lines(tmp_path: Path) -> None:
    trace_path = tmp_path / "turns.jsonl"
    trace_path.write_text(
        "not-json\n" + json.dumps({"turn_id": "turn-1", "outcome": "ok"}) + "\n",
        encoding="utf-8",
    )

    runner = CliRunner()
    result = runner.invoke(app, ["voice", "trace", "last", "--path", str(trace_path)])

    assert result.exit_code == 0, result.output
    assert "turn-1" in result.output
    assert "not-json" not in result.output
```

In `tests/cli/test_yoyopod_cli_docgen.py`, add:

```python
def test_docgen_contains_voice_commands() -> None:
    md = generate_commands_md(app)

    assert "## `yoyopod voice" in md
    assert "`yoyopod voice trace last`" in md
```

- [ ] **Step 2: Run CLI tests to verify they fail**

Run:

```bash
uv run pytest tests/cli/test_yoyopod_cli_voice.py tests/cli/test_yoyopod_cli_docgen.py::test_docgen_contains_voice_commands -q
```

Expected: FAIL because the `voice` Typer group does not exist.

- [ ] **Step 3: Implement the voice CLI trace group**

Create `yoyopod_cli/voice.py`:

```python
"""Voice diagnostics CLI commands."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import typer

from yoyopod.integrations.voice.trace import DEFAULT_VOICE_TRACE_PATH, VoiceTraceStore

app = typer.Typer(name="voice", help="Voice diagnostics and validation.")
trace_app = typer.Typer(name="trace", help="Inspect local voice trace entries.")

app.add_typer(trace_app, name="trace")


def _cell(value: object, *, width: int = 28) -> str:
    text = "" if value is None else " ".join(str(value).split())
    if len(text) <= width:
        return text
    return text[: width - 3] + "..."


def _row(payload: dict[str, Any]) -> str:
    return (
        f"{_cell(payload.get('completed_at'), width=24)}  "
        f"{_cell(payload.get('turn_id'), width=16)}  "
        f"{_cell(payload.get('source'), width=12)}  "
        f"{_cell(payload.get('mode'), width=8)}  "
        f"{_cell(payload.get('route_kind'), width=14)}  "
        f"{_cell(payload.get('outcome'), width=18)}  "
        f"{_cell(payload.get('command_intent') or payload.get('route_name'), width=18)}  "
        f"{_cell(payload.get('transcript_normalized') or payload.get('assistant_body_preview'), width=40)}"
    )


@trace_app.command("last")
def trace_last(
    limit: int = typer.Option(5, "--limit", min=1, help="Number of recent turns to print."),
    path: Path = typer.Option(
        DEFAULT_VOICE_TRACE_PATH,
        "--path",
        help="Voice trace JSONL path.",
    ),
) -> None:
    """Print recent voice trace entries, newest first."""

    store = VoiceTraceStore(path=path, max_turns=max(1, limit))
    entries = list(reversed(store.read_recent(limit=limit)))
    if not entries:
        typer.echo(f"No voice trace entries at {path}")
        return
    typer.echo(
        "completed_at              turn_id           source        mode      route_kind      "
        "outcome             target             text"
    )
    for payload in entries:
        typer.echo(_row(payload))

```

In `yoyopod_cli/main.py`, add after the health group:

```python
from yoyopod_cli import voice as _voice  # noqa: E402

app.add_typer(_voice.app, name="voice")
```

- [ ] **Step 4: Run CLI trace tests**

Run:

```bash
uv run pytest tests/cli/test_yoyopod_cli_voice.py::test_voice_group_is_registered tests/cli/test_yoyopod_cli_voice.py::test_voice_trace_last_prints_recent_rows tests/cli/test_yoyopod_cli_voice.py::test_voice_trace_last_tolerates_missing_file tests/cli/test_yoyopod_cli_voice.py::test_voice_trace_last_ignores_corrupt_lines -q
```

Expected: PASS for trace CLI tests.

- [ ] **Step 5: Regenerate command docs**

Run:

```bash
uv run python -m yoyopod_cli.main dev docs
uv run pytest tests/cli/test_yoyopod_cli_docgen.py::test_docgen_contains_voice_commands tests/cli/test_yoyopod_cli_docs_drift.py -q
```

Expected: PASS and `yoyopod_cli/COMMANDS.md` contains `yoyopod voice trace last`.

- [ ] **Step 6: Run required gates and commit Task 2**

Run:

```bash
uv run python scripts/quality.py gate
uv run pytest -q
git add yoyopod_cli/voice.py yoyopod_cli/main.py yoyopod_cli/COMMANDS.md tests/cli/test_yoyopod_cli_voice.py tests/cli/test_yoyopod_cli_docgen.py
git commit -m "feat: add voice trace cli"
```

Expected: quality passes, full pytest passes, commit created.

## Task 3: Dictionary Validator And CLI Validation

**Files:**
- Create: `yoyopod/integrations/voice/dictionary_validator.py`
- Create: `tests/integrations/test_voice_dictionary_validator.py`
- Modify: `yoyopod_cli/voice.py`
- Modify: `tests/cli/test_yoyopod_cli_voice.py`
- Modify: `tests/cli/test_yoyopod_cli_docgen.py`
- Modify: `yoyopod_cli/COMMANDS.md`

- [ ] **Step 1: Add failing validator unit tests**

Create `tests/integrations/test_voice_dictionary_validator.py`:

```python
"""Tests for strict voice command dictionary validation."""

from __future__ import annotations

from pathlib import Path

import yaml

from yoyopod.integrations.voice.dictionary_validator import (
    validate_voice_command_dictionary,
)


def _write_yaml(path: Path, payload: object) -> None:
    path.write_text(yaml.safe_dump(payload, sort_keys=False), encoding="utf-8")


def test_dictionary_validator_accepts_matching_examples(tmp_path: Path) -> None:
    path = tmp_path / "commands.yaml"
    _write_yaml(
        path,
        {
            "version": 1,
            "intents": {
                "call_contact": {
                    "aliases": ["call mama"],
                    "examples": ["call mama", "please call mama"],
                },
                "volume_up": {
                    "aliases": ["boost sound"],
                    "examples": ["boost sound"],
                },
            },
            "actions": {
                "open_talk": {
                    "aliases": ["open talk"],
                    "route": "open_talk",
                }
            },
        },
    )

    result = validate_voice_command_dictionary(path)

    assert result.errors == []
    assert result.has_errors is False


def test_dictionary_validator_reports_bad_example_intent(tmp_path: Path) -> None:
    path = tmp_path / "commands.yaml"
    _write_yaml(
        path,
        {
            "version": 1,
            "intents": {
                "volume_up": {
                    "aliases": ["boost sound"],
                    "examples": ["call mama"],
                }
            },
        },
    )

    result = validate_voice_command_dictionary(path)

    assert result.has_errors is True
    assert any("expected volume_up" in issue.message for issue in result.errors)


def test_dictionary_validator_reports_unsafe_action(tmp_path: Path) -> None:
    path = tmp_path / "commands.yaml"
    _write_yaml(
        path,
        {
            "version": 1,
            "actions": {
                "shell": {
                    "aliases": ["run update"],
                    "route": "powershell",
                }
            },
        },
    )

    result = validate_voice_command_dictionary(path)

    assert any("unsafe route" in issue.message for issue in result.errors)


def test_dictionary_validator_reports_duplicate_alias_across_intents(tmp_path: Path) -> None:
    path = tmp_path / "commands.yaml"
    _write_yaml(
        path,
        {
            "version": 1,
            "intents": {
                "volume_up": {"aliases": ["boost sound"]},
                "play_music": {"aliases": ["boost sound"]},
            },
        },
    )

    result = validate_voice_command_dictionary(path)

    assert any("duplicate alias" in issue.message for issue in result.errors)


def test_dictionary_validator_reports_yaml_parse_error(tmp_path: Path) -> None:
    path = tmp_path / "commands.yaml"
    path.write_text("intents: [", encoding="utf-8")

    result = validate_voice_command_dictionary(path)

    assert result.has_errors is True
    assert "YAML" in result.errors[0].message


def test_dictionary_validator_warns_for_short_alias(tmp_path: Path) -> None:
    path = tmp_path / "commands.yaml"
    _write_yaml(
        path,
        {
            "version": 1,
            "intents": {
                "volume_up": {
                    "aliases": ["up"],
                    "examples": ["volume up"],
                }
            },
        },
    )

    result = validate_voice_command_dictionary(path)

    assert result.errors == []
    assert any("short" in issue.message for issue in result.warnings)
```

- [ ] **Step 2: Run validator tests to verify they fail**

Run:

```bash
uv run pytest tests/integrations/test_voice_dictionary_validator.py -q
```

Expected: FAIL because `yoyopod.integrations.voice.dictionary_validator` does not exist.

- [ ] **Step 3: Implement the dictionary validator**

Create `yoyopod/integrations/voice/dictionary_validator.py`:

```python
"""Strict validation for mutable voice command dictionaries."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml

from yoyopod.integrations.voice.commands import VoiceCommandIntent, match_voice_command
from yoyopod.integrations.voice.dictionary import (
    SAFE_VOICE_ROUTE_ACTIONS,
    VoiceCommandDictionary,
    _merge_dictionary_payload,
)


@dataclass(slots=True, frozen=True)
class DictionaryValidationIssue:
    """One dictionary validation issue."""

    location: str
    message: str


@dataclass(slots=True, frozen=True)
class DictionaryValidationResult:
    """Dictionary validation result."""

    errors: tuple[DictionaryValidationIssue, ...]
    warnings: tuple[DictionaryValidationIssue, ...]

    @property
    def has_errors(self) -> bool:
        return bool(self.errors)

    @property
    def has_warnings(self) -> bool:
        return bool(self.warnings)


def validate_voice_command_dictionary(
    path: str | Path,
    *,
    allow_missing: bool = False,
) -> DictionaryValidationResult:
    """Validate one mutable voice command dictionary YAML file."""

    dictionary_path = Path(path)
    errors: list[DictionaryValidationIssue] = []
    warnings: list[DictionaryValidationIssue] = []
    try:
        payload = yaml.safe_load(dictionary_path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        if allow_missing:
            return DictionaryValidationResult(errors=(), warnings=())
        return DictionaryValidationResult(
            errors=(DictionaryValidationIssue(str(dictionary_path), "dictionary file not found"),),
            warnings=(),
        )
    except yaml.YAMLError as exc:
        return DictionaryValidationResult(
            errors=(DictionaryValidationIssue(str(dictionary_path), f"YAML parse error: {exc}"),),
            warnings=(),
        )
    except OSError as exc:
        return DictionaryValidationResult(
            errors=(DictionaryValidationIssue(str(dictionary_path), f"cannot read file: {exc}"),),
            warnings=(),
        )

    if payload is None:
        payload = {}
    if not isinstance(payload, dict):
        return DictionaryValidationResult(
            errors=(DictionaryValidationIssue(str(dictionary_path), "root must be a mapping"),),
            warnings=(),
        )

    _validate_intents(payload.get("intents"), errors, warnings)
    _validate_actions(payload.get("actions"), errors, warnings)
    _validate_duplicate_aliases(payload, errors)

    if not errors:
        dictionary = _merge_dictionary_payload(payload)
        _validate_examples(payload.get("intents"), dictionary, errors)

    return DictionaryValidationResult(errors=tuple(errors), warnings=tuple(warnings))


def _validate_intents(
    payload: Any,
    errors: list[DictionaryValidationIssue],
    warnings: list[DictionaryValidationIssue],
) -> None:
    if payload is None:
        return
    if not isinstance(payload, dict):
        errors.append(DictionaryValidationIssue("intents", "intents must be a mapping"))
        return
    valid_intents = {intent.value for intent in VoiceCommandIntent if intent is not VoiceCommandIntent.UNKNOWN}
    for intent_name, intent_payload in payload.items():
        location = f"intents.{intent_name}"
        if not isinstance(intent_name, str) or intent_name not in valid_intents:
            errors.append(DictionaryValidationIssue(location, "unknown voice command intent"))
            continue
        if not isinstance(intent_payload, dict):
            errors.append(DictionaryValidationIssue(location, "intent entry must be a mapping"))
            continue
        for field_name in ("aliases", "examples"):
            values = intent_payload.get(field_name)
            if values is None:
                continue
            if not _is_string_list(values):
                errors.append(DictionaryValidationIssue(f"{location}.{field_name}", "must be a string or list of strings"))
                continue
            _warn_short_phrases(f"{location}.{field_name}", _string_tuple(values), warnings)
        if "fuzzy_threshold" in intent_payload:
            threshold = intent_payload["fuzzy_threshold"]
            if not isinstance(threshold, int | float) or not 0.0 <= float(threshold) <= 1.0:
                errors.append(DictionaryValidationIssue(f"{location}.fuzzy_threshold", "must be between 0.0 and 1.0"))


def _validate_actions(
    payload: Any,
    errors: list[DictionaryValidationIssue],
    warnings: list[DictionaryValidationIssue],
) -> None:
    if payload is None:
        return
    if not isinstance(payload, dict):
        errors.append(DictionaryValidationIssue("actions", "actions must be a mapping"))
        return
    for action_name, action_payload in payload.items():
        location = f"actions.{action_name}"
        if not isinstance(action_name, str) or not action_name.strip():
            errors.append(DictionaryValidationIssue(location, "action name must be a non-empty string"))
            continue
        if not isinstance(action_payload, dict):
            errors.append(DictionaryValidationIssue(location, "action entry must be a mapping"))
            continue
        route = action_payload.get("route")
        if not isinstance(route, str) or route not in SAFE_VOICE_ROUTE_ACTIONS:
            errors.append(DictionaryValidationIssue(f"{location}.route", f"unsafe route action: {route!r}"))
        aliases = action_payload.get("aliases")
        if not _is_string_list(aliases):
            errors.append(DictionaryValidationIssue(f"{location}.aliases", "must be a string or list of strings"))
        else:
            _warn_short_phrases(f"{location}.aliases", _string_tuple(aliases), warnings)


def _validate_examples(
    payload: Any,
    dictionary: VoiceCommandDictionary,
    errors: list[DictionaryValidationIssue],
) -> None:
    if not isinstance(payload, dict):
        return
    grammar = dictionary.to_grammar()
    for intent_name, intent_payload in payload.items():
        if not isinstance(intent_name, str) or not isinstance(intent_payload, dict):
            continue
        if intent_name not in {intent.value for intent in VoiceCommandIntent}:
            continue
        expected = VoiceCommandIntent(intent_name)
        for index, example in enumerate(_string_tuple(intent_payload.get("examples"))):
            match = match_voice_command(example, grammar=grammar)
            if match.intent is not expected:
                errors.append(
                    DictionaryValidationIssue(
                        f"intents.{intent_name}.examples[{index}]",
                        f"example {example!r} matched {match.intent.value}; expected {intent_name}",
                    )
                )


def _validate_duplicate_aliases(
    payload: dict[Any, Any],
    errors: list[DictionaryValidationIssue],
) -> None:
    owners: dict[str, str] = {}
    for intent_name, intent_payload in _mapping_items(payload.get("intents")):
        if not isinstance(intent_payload, dict):
            continue
        for alias in _string_tuple(intent_payload.get("aliases")):
            _claim_alias(_normalize_phrase(alias), f"intent {intent_name}", owners, errors)
    for action_name, action_payload in _mapping_items(payload.get("actions")):
        if not isinstance(action_payload, dict):
            continue
        for alias in _string_tuple(action_payload.get("aliases")):
            _claim_alias(_normalize_phrase(alias), f"action {action_name}", owners, errors)


def _claim_alias(
    alias: str,
    owner: str,
    owners: dict[str, str],
    errors: list[DictionaryValidationIssue],
) -> None:
    if not alias:
        return
    existing_owner = owners.get(alias)
    if existing_owner is not None and existing_owner != owner:
        errors.append(DictionaryValidationIssue(alias, f"duplicate alias claimed by {existing_owner} and {owner}"))
        return
    owners[alias] = owner


def _mapping_items(payload: Any) -> tuple[tuple[Any, Any], ...]:
    if not isinstance(payload, dict):
        return ()
    return tuple(payload.items())


def _is_string_list(value: Any) -> bool:
    if isinstance(value, str):
        return True
    return isinstance(value, list | tuple) and all(isinstance(item, str) for item in value)


def _string_tuple(value: Any) -> tuple[str, ...]:
    if isinstance(value, str):
        stripped = value.strip()
        return (stripped,) if stripped else ()
    if not isinstance(value, list | tuple):
        return ()
    return tuple(stripped for item in value if isinstance(item, str) if (stripped := item.strip()))


def _normalize_phrase(value: str) -> str:
    return " ".join(value.strip().lower().split())


def _warn_short_phrases(
    location: str,
    phrases: tuple[str, ...],
    warnings: list[DictionaryValidationIssue],
) -> None:
    allowed_short = {"play", "louder", "quieter"}
    for phrase in phrases:
        normalized = _normalize_phrase(phrase)
        if normalized in allowed_short:
            continue
        if len(normalized.split()) < 2:
            warnings.append(DictionaryValidationIssue(location, f"short alias or example may be ambiguous: {phrase!r}"))
```

- [ ] **Step 4: Run validator tests**

Run:

```bash
uv run pytest tests/integrations/test_voice_dictionary_validator.py -q
```

Expected: PASS.

- [ ] **Step 5: Add dictionary CLI tests**

In `tests/cli/test_yoyopod_cli_docgen.py`, update `test_docgen_contains_voice_commands`:

```python
def test_docgen_contains_voice_commands() -> None:
    md = generate_commands_md(app)

    assert "## `yoyopod voice" in md
    assert "`yoyopod voice trace last`" in md
    assert "`yoyopod voice dictionary validate`" in md
```

Append these tests to `tests/cli/test_yoyopod_cli_voice.py`:

```python
def test_voice_dictionary_validate_accepts_valid_file(tmp_path: Path) -> None:
    path = tmp_path / "commands.yaml"
    path.write_text(
        "version: 1\nintents:\n  volume_up:\n    aliases: [boost sound]\n    examples: [boost sound]\n",
        encoding="utf-8",
    )

    runner = CliRunner()
    result = runner.invoke(app, ["voice", "dictionary", "validate", "--path", str(path)])

    assert result.exit_code == 0, result.output
    assert "OK voice dictionary" in result.output


def test_voice_dictionary_validate_fails_on_errors(tmp_path: Path) -> None:
    path = tmp_path / "commands.yaml"
    path.write_text(
        "version: 1\nactions:\n  shell:\n    aliases: [run update]\n    route: powershell\n",
        encoding="utf-8",
    )

    runner = CliRunner()
    result = runner.invoke(app, ["voice", "dictionary", "validate", "--path", str(path)])

    assert result.exit_code == 1
    assert "unsafe route" in result.output


def test_voice_dictionary_validate_strict_fails_on_warnings(tmp_path: Path) -> None:
    path = tmp_path / "commands.yaml"
    path.write_text(
        "version: 1\nintents:\n  volume_up:\n    aliases: [up]\n    examples: [volume up]\n",
        encoding="utf-8",
    )

    runner = CliRunner()
    result = runner.invoke(
        app,
        ["voice", "dictionary", "validate", "--path", str(path), "--strict"],
    )

    assert result.exit_code == 1
    assert "WARN" in result.output


def test_voice_dictionary_validate_default_missing_uses_builtins(monkeypatch, tmp_path: Path) -> None:
    monkeypatch.chdir(tmp_path)

    runner = CliRunner()
    result = runner.invoke(app, ["voice", "dictionary", "validate"])

    assert result.exit_code == 0, result.output
    assert "built-ins only" in result.output
```

- [ ] **Step 6: Add dictionary CLI implementation**

In `yoyopod_cli/voice.py`, add a dictionary subapp near the trace subapp:

```python
dictionary_app = typer.Typer(name="dictionary", help="Validate voice command dictionaries.")
app.add_typer(dictionary_app, name="dictionary")
```

Add this command after `trace_last`:

```python
@dictionary_app.command("validate")
def dictionary_validate(
    path: Path | None = typer.Option(
        None,
        "--path",
        help="Voice command dictionary YAML path.",
    ),
    strict: bool = typer.Option(False, "--strict", help="Treat warnings as failures."),
) -> None:
    """Validate a voice command dictionary."""

    from yoyopod.integrations.voice.dictionary_validator import (
        validate_voice_command_dictionary,
    )

    dictionary_path = path or Path("data/voice/commands.yaml")
    result = validate_voice_command_dictionary(
        dictionary_path,
        allow_missing=path is None,
    )
    for issue in result.errors:
        typer.echo(f"ERROR {issue.location}: {issue.message}", err=True)
    for issue in result.warnings:
        typer.echo(f"WARN {issue.location}: {issue.message}")
    if result.has_errors or (strict and result.has_warnings):
        raise typer.Exit(code=1)
    suffix = " (built-ins only)" if path is None and not dictionary_path.exists() else ""
    typer.echo(f"OK voice dictionary {dictionary_path}{suffix}")
```

- [ ] **Step 7: Run dictionary CLI tests and docs drift tests**

Run:

```bash
uv run python -m yoyopod_cli.main dev docs
uv run pytest tests/cli/test_yoyopod_cli_voice.py::test_voice_dictionary_validate_accepts_valid_file tests/cli/test_yoyopod_cli_voice.py::test_voice_dictionary_validate_fails_on_errors tests/cli/test_yoyopod_cli_voice.py::test_voice_dictionary_validate_strict_fails_on_warnings tests/cli/test_yoyopod_cli_voice.py::test_voice_dictionary_validate_default_missing_uses_builtins tests/cli/test_yoyopod_cli_docgen.py::test_docgen_contains_voice_commands tests/cli/test_yoyopod_cli_docs_drift.py -q
```

Expected: PASS.

- [ ] **Step 8: Run required gates and commit Task 3**

Run:

```bash
uv run python scripts/quality.py gate
uv run pytest -q
git add yoyopod/integrations/voice/dictionary_validator.py yoyopod_cli/voice.py yoyopod_cli/COMMANDS.md tests/integrations/test_voice_dictionary_validator.py tests/cli/test_yoyopod_cli_voice.py tests/cli/test_yoyopod_cli_docgen.py
git commit -m "feat: validate voice command dictionaries"
```

Expected: quality passes, full pytest passes, commit created.

## Task 4: Runtime Trace Integration

**Files:**
- Modify: `yoyopod/integrations/voice/trace.py`
- Modify: `yoyopod/integrations/voice/runtime.py`
- Modify: `yoyopod/core/bootstrap/screens_boot.py`
- Modify: `tests/integrations/test_voice_runtime.py`

- [ ] **Step 1: Add failing runtime trace tests**

Append these helpers near the fake classes in `tests/integrations/test_voice_runtime.py`:

```python
class _MemoryTraceStore:
    def __init__(self) -> None:
        self.entries: list[object] = []

    def append(self, entry: object) -> None:
        self.entries.append(entry)
```

Append these tests after the Ask routing tests:

```python
def test_begin_ask_traces_command_turn() -> None:
    context = AppContext()
    service = _FakeVoiceService("hey yoyo play music")
    trace_store = _MemoryTraceStore()
    coordinator = VoiceRuntimeCoordinator(
        context=context,
        settings_resolver=VoiceSettingsResolver(
            context=context,
            settings_provider=lambda: VoiceSettings(
                mode="cloud",
                activation_prefixes=("hey yoyo", "yoyo"),
                ask_fallback_enabled=True,
                voice_trace_enabled=True,
                voice_trace_include_transcripts=True,
            ),
        ),
        command_executor=_build_executor(context=context, play_music_action=lambda: True),
        voice_service_factory=lambda settings: service,
        output_player=_FakeOutputPlayer(),
        ask_client=_FakeAskClient(["unused"]),
        trace_store_factory=lambda settings: trace_store,
    )

    coordinator.begin_ask(async_capture=False)

    assert len(trace_store.entries) == 1
    payload = trace_store.entries[0].to_json_dict()
    assert payload["source"] == "ask_screen"
    assert payload["mode"] == "ask"
    assert payload["route_kind"] == "command"
    assert payload["transcript_normalized"] == "play music"
    assert payload["command_intent"] == "play_music"
    assert payload["outcome"] == "Playing"


def test_begin_ask_traces_ask_fallback_with_capped_body_preview() -> None:
    context = AppContext()
    service = _FakeVoiceService("hey yoyo why is the sky blue")
    trace_store = _MemoryTraceStore()
    coordinator = VoiceRuntimeCoordinator(
        context=context,
        settings_resolver=VoiceSettingsResolver(
            context=context,
            settings_provider=lambda: VoiceSettings(
                mode="cloud",
                activation_prefixes=("hey yoyo", "yoyo"),
                ask_fallback_enabled=True,
                voice_trace_body_preview_chars=12,
            ),
        ),
        command_executor=_build_executor(context=context),
        voice_service_factory=lambda settings: service,
        output_player=_FakeOutputPlayer(),
        ask_client=_FakeAskClient(["Because sunlight scatters in the air."]),
        trace_store_factory=lambda settings: trace_store,
    )

    coordinator.begin_ask(async_capture=False)
    coordinator._tts_queue.join()

    payload = trace_store.entries[0].to_json_dict()
    assert payload["route_kind"] == "ask"
    assert payload["transcript_normalized"] == "why is the sky blue"
    assert payload["assistant_body_preview"] == "Because s..."


def test_begin_listening_traces_stt_failure() -> None:
    class _FailingTranscribeVoiceService(_FakeVoiceService):
        def transcribe(
            self,
            audio_path: Path,
            *,
            cancel_event: threading.Event | None = None,
        ) -> VoiceTranscript:
            raise RuntimeError("stt boom")

    context = AppContext()
    trace_store = _MemoryTraceStore()
    coordinator = VoiceRuntimeCoordinator(
        context=context,
        settings_resolver=VoiceSettingsResolver(
            context=context,
            settings_provider=lambda: VoiceSettings(voice_trace_enabled=True),
        ),
        command_executor=_build_executor(context=context),
        voice_service_factory=lambda settings: _FailingTranscribeVoiceService(""),
        output_player=_FakeOutputPlayer(),
        trace_store_factory=lambda settings: trace_store,
    )

    coordinator.begin_listening(async_capture=False)

    payload = trace_store.entries[0].to_json_dict()
    assert payload["route_kind"] == "error"
    assert payload["outcome"] == "Mic Unavailable"
    assert payload["error"]["stage"] == "stt"
    assert payload["error"]["type"] == "RuntimeError"


def test_begin_ask_trace_records_music_focus_after_resume() -> None:
    context = AppContext()
    service = _FakeVoiceService("hey yoyo play music")
    music_backend = _FakeMusicBackend("playing")
    trace_store = _MemoryTraceStore()
    coordinator = VoiceRuntimeCoordinator(
        context=context,
        settings_resolver=VoiceSettingsResolver(
            context=context,
            settings_provider=lambda: VoiceSettings(
                activation_prefixes=("hey yoyo", "yoyo"),
                ask_fallback_enabled=True,
            ),
        ),
        command_executor=_build_executor(context=context, play_music_action=lambda: True),
        voice_service_factory=lambda settings: service,
        output_player=_FakeOutputPlayer(),
        ask_client=_FakeAskClient(["unused"]),
        music_backend=music_backend,
        trace_store_factory=lambda settings: trace_store,
    )

    coordinator.begin_ask(async_capture=False)

    payload = trace_store.entries[0].to_json_dict()
    assert payload["music_before"]["playback_state"] == "playing"
    assert payload["audio_focus_before"]["music_paused_for_voice"] is False
    assert payload["music_after"]["playback_state"] == "playing"
    assert payload["audio_focus_after"]["music_paused_for_voice"] is False
```

- [ ] **Step 2: Run runtime trace tests to verify they fail**

Run:

```bash
uv run pytest tests/integrations/test_voice_runtime.py::test_begin_ask_traces_command_turn tests/integrations/test_voice_runtime.py::test_begin_ask_traces_ask_fallback_with_capped_body_preview tests/integrations/test_voice_runtime.py::test_begin_listening_traces_stt_failure tests/integrations/test_voice_runtime.py::test_begin_ask_trace_records_music_focus_after_resume -q
```

Expected: FAIL because `VoiceRuntimeCoordinator` does not accept `trace_store_factory` and does not trace turns.

- [ ] **Step 3: Add a mutable recorder to trace.py**

Append this class to `yoyopod/integrations/voice/trace.py`:

```python
@dataclass(slots=True)
class VoiceTraceRecorder:
    """Mutable recorder that writes one trace entry on completion."""

    store: Any
    turn_id: str
    started_at: str
    source: str
    mode: str
    include_transcripts: bool = True
    body_preview_chars: int = 160
    route_kind: str = "unknown"
    outcome: str = "unknown"
    transcript_raw: str | None = None
    transcript_normalized: str | None = None
    activation_prefix: str | None = None
    command_intent: str | None = None
    command_confidence: float | None = None
    route_name: str | None = None
    ask_fallback: bool | None = None
    assistant_status: str | None = None
    assistant_title: str | None = None
    assistant_body_preview: str | None = None
    should_speak: bool | None = None
    auto_return: bool | None = None
    timings_ms: dict[str, float] = field(default_factory=dict)
    audio_focus_before: dict[str, Any] = field(default_factory=dict)
    audio_focus_after: dict[str, Any] = field(default_factory=dict)
    music_before: dict[str, Any] = field(default_factory=dict)
    music_after: dict[str, Any] = field(default_factory=dict)
    error: dict[str, str] | None = None
    _completed: bool = False

    def record_error(self, *, stage: str, exc: BaseException) -> None:
        self.route_kind = "error"
        self.error = {
            "stage": stage,
            "type": type(exc).__name__,
            "message": str(exc),
        }

    def complete(self) -> None:
        if self._completed:
            return
        self._completed = True
        self.store.append(
            VoiceTraceEntry(
                turn_id=self.turn_id,
                started_at=self.started_at,
                completed_at=utc_now_iso(),
                source=self.source,
                mode=self.mode,
                route_kind=self.route_kind,
                outcome=self.outcome,
                transcript_raw=self.transcript_raw,
                transcript_normalized=self.transcript_normalized,
                activation_prefix=self.activation_prefix,
                command_intent=self.command_intent,
                command_confidence=self.command_confidence,
                route_name=self.route_name,
                ask_fallback=self.ask_fallback,
                assistant_status=self.assistant_status,
                assistant_title=self.assistant_title,
                assistant_body_preview=self.assistant_body_preview,
                should_speak=self.should_speak,
                auto_return=self.auto_return,
                timings_ms=self.timings_ms,
                audio_focus_before=self.audio_focus_before,
                audio_focus_after=self.audio_focus_after,
                music_before=self.music_before,
                music_after=self.music_after,
                error=self.error,
                include_transcripts=self.include_transcripts,
                body_preview_chars=self.body_preview_chars,
            )
        )
```

- [ ] **Step 4: Wire trace recorder into VoiceRuntimeCoordinator**

In `yoyopod/integrations/voice/runtime.py`, add imports:

```python
from yoyopod.integrations.voice.commands import match_voice_command
from yoyopod.integrations.voice.trace import (
    VoiceTraceRecorder,
    VoiceTraceStore,
    new_turn_id,
    utc_now_iso,
)
```

Update `TYPE_CHECKING` import block to keep only `VoiceCommandMatch` there if needed.

Add this parameter to `VoiceRuntimeCoordinator.__init__`:

```python
        trace_store_factory: Callable[[VoiceSettings], VoiceTraceStore | None] | None = None,
```

Set these fields in `__init__`:

```python
        self._trace_store_factory = trace_store_factory
        self._trace_lock = threading.Lock()
        self._active_traces: dict[int, VoiceTraceRecorder] = {}
```

Add these helper methods near `_voice_router`:

```python
    def _begin_trace(
        self,
        *,
        settings: VoiceSettings,
        generation: int,
        source: str,
        mode: str,
    ) -> None:
        if not settings.voice_trace_enabled or self._trace_store_factory is None:
            return
        try:
            store = self._trace_store_factory(settings)
        except Exception as exc:
            logger.debug("Voice trace store unavailable: {}", exc)
            return
        if store is None:
            return
        recorder = VoiceTraceRecorder(
            store=store,
            turn_id=new_turn_id(),
            started_at=utc_now_iso(),
            source=source,
            mode=mode,
            include_transcripts=settings.voice_trace_include_transcripts,
            body_preview_chars=settings.voice_trace_body_preview_chars,
            audio_focus_before=self._audio_focus_snapshot(),
            music_before=self._music_snapshot(),
        )
        with self._trace_lock:
            self._active_traces[generation] = recorder

    def _trace_for_generation(self, generation: int | None = None) -> VoiceTraceRecorder | None:
        if generation is None:
            generation = self._state.generation
        with self._trace_lock:
            return self._active_traces.get(generation)

    def _complete_trace(self, generation: int | None = None) -> None:
        if generation is None:
            generation = self._state.generation
        with self._trace_lock:
            recorder = self._active_traces.pop(generation, None)
        if recorder is None:
            return
        recorder.audio_focus_after = self._audio_focus_snapshot()
        recorder.music_after = self._music_snapshot()
        try:
            recorder.complete()
        except Exception as exc:
            logger.debug("Voice trace completion failed: {}", exc)

    def _audio_focus_snapshot(self) -> dict[str, object]:
        with self._music_focus_lock:
            return {
                "music_paused_for_voice": self._music_paused_for_voice,
                "music_paused_generation": self._music_paused_generation,
            }

    def _music_snapshot(self) -> dict[str, object]:
        music_backend = self._music_backend
        if music_backend is None:
            return {"connected": False}
        connected = bool(getattr(music_backend, "is_connected", False))
        payload: dict[str, object] = {"connected": connected}
        if not connected:
            return payload
        try:
            payload["playback_state"] = music_backend.get_playback_state()
        except Exception as exc:
            payload["playback_state_error"] = str(exc)
        return payload
```

In `begin_ask`, after `generation = self._next_generation()` and before `_pause_music_for_voice(...)`, add:

```python
        self._begin_trace(
            settings=settings,
            generation=generation,
            source="ask_screen",
            mode="ask",
        )
```

In `begin_listening`, after `generation = self._next_generation()` and before `_pause_music_for_voice(...)`, add:

```python
        self._begin_trace(
            settings=settings,
            generation=generation,
            source="ask_screen",
            mode="command",
        )
```

In `begin_ptt_capture`, after `generation = self._next_generation()` and before `_pause_music_for_voice(...)`, add:

```python
        self._begin_trace(
            settings=settings,
            generation=generation,
            source="hub_hold",
            mode="ptt",
        )
```

In `_run_listening_cycle`, after successful transcription and before `dispatch_listen_result(...)`, add:

```python
        recorder = self._trace_for_generation(generation)
        if recorder is not None:
            recorder.transcript_raw = transcript.text.strip()
            recorder.transcript_normalized = transcript.text.strip()
```

In `_run_listening_cycle`, inside the transcription exception block before dispatching failure, add:

```python
            recorder = self._trace_for_generation(generation)
            if recorder is not None:
                recorder.record_error(stage="stt", exc=exc)
```

In `_run_ask_cycle`, after `question = transcript.text.strip()` and before routing, add:

```python
        recorder = self._trace_for_generation(generation)
        if recorder is not None:
            recorder.transcript_raw = transcript.text.strip()
```

After `decision = router.route(question)`, add:

```python
        recorder = self._trace_for_generation(generation)
        if recorder is not None:
            recorder.transcript_normalized = decision.normalized_text
            recorder.activation_prefix = decision.stripped_prefix or None
            recorder.command_confidence = decision.confidence
            recorder.route_name = decision.route_name
            recorder.ask_fallback = decision.kind is VoiceRouteKind.ASK_FALLBACK
            if decision.kind is VoiceRouteKind.COMMAND and decision.command is not None:
                recorder.route_kind = "command"
                recorder.command_intent = decision.command.intent.value
            elif decision.kind is VoiceRouteKind.ACTION:
                recorder.route_kind = "command"
            elif decision.kind is VoiceRouteKind.ASK_FALLBACK:
                recorder.route_kind = "ask"
            elif decision.kind is VoiceRouteKind.LOCAL_HELP:
                recorder.route_kind = "silence"
```

In `_run_ask_cycle`, inside the STT exception block before dispatching failure, add:

```python
            recorder = self._trace_for_generation(generation)
            if recorder is not None:
                recorder.record_error(stage="stt", exc=exc)
```

In `_execute_command_transcript`, replace the initial command execution with:

```python
        if command is None:
            command = match_voice_command(transcript)
        outcome = self._command_executor.execute(transcript, command=command)
        recorder = self._trace_for_generation()
        if recorder is not None:
            recorder.route_kind = "command" if command.is_command else "silence"
            recorder.command_intent = command.intent.value if command.is_command else None
            recorder.transcript_normalized = transcript.strip()
```

In `_apply_outcome`, before `_set_state(...)`, add:

```python
        recorder = self._trace_for_generation()
        if recorder is not None:
            recorder.outcome = outcome.headline
            recorder.assistant_status = outcome.headline
            recorder.assistant_title = outcome.headline
            recorder.assistant_body_preview = outcome.body
            recorder.should_speak = outcome.should_speak
            recorder.auto_return = outcome.auto_return
            recorder.route_name = outcome.route_name or recorder.route_name
            if recorder.error is not None:
                recorder.route_kind = "error"
```

Change the `_speak_outcome_async` call in `_apply_outcome` to pass the current generation:

```python
            self._speak_outcome_async(
                outcome.body,
                generation=self._state.generation,
                release_music_after=outcome,
            )
```

After `_resume_music_after_voice(outcome)` in the non-speaking branch of `_apply_outcome`, add:

```python
            self._complete_trace()
```

In `_apply_ask_outcome`, before `_set_state(...)`, add the same recorder field assignment used in `_apply_outcome`, and keep `route_kind` as `"ask"` when `outcome.headline == "Answer"` and no error is present.

After `_resume_music_after_voice(outcome)` in the non-speaking branch of `_apply_ask_outcome`, add:

```python
            self._complete_trace(generation)
```

In `_run_tts_worker`, after `_resume_music_after_voice(item.release_music_after)`, add:

```python
                    self._complete_trace(item.generation)
```

In `cancel`, before the final `_set_state(...)`, add:

```python
        recorder = self._trace_for_generation()
        if recorder is not None:
            recorder.route_kind = "error"
            recorder.outcome = "cancelled"
        self._complete_trace()
```

- [ ] **Step 5: Pass trace store factory from canonical app boot**

In `yoyopod/core/bootstrap/screens_boot.py`, add:

```python
from yoyopod.integrations.voice.trace import VoiceTraceStore
```

In the canonical `VoiceRuntimeCoordinator(...)` construction, add:

```python
                trace_store_factory=VoiceTraceStore.from_settings,
```

Do not pass a trace factory from tests unless a test explicitly wants trace writes. This keeps existing direct coordinator tests from writing `logs/voice/turns.jsonl`.

- [ ] **Step 6: Run runtime trace tests**

Run:

```bash
uv run pytest tests/integrations/test_voice_runtime.py::test_begin_ask_traces_command_turn tests/integrations/test_voice_runtime.py::test_begin_ask_traces_ask_fallback_with_capped_body_preview tests/integrations/test_voice_runtime.py::test_begin_listening_traces_stt_failure tests/integrations/test_voice_runtime.py::test_begin_ask_trace_records_music_focus_after_resume -q
```

Expected: PASS.

- [ ] **Step 7: Run focused voice runtime regression tests**

Run:

```bash
uv run pytest tests/integrations/test_voice_runtime.py tests/integrations/test_voice_dictionary.py tests/integrations/test_voice_router.py -q
```

Expected: PASS.

- [ ] **Step 8: Run required gates and commit Task 4**

Run:

```bash
uv run python scripts/quality.py gate
uv run pytest -q
git add yoyopod/integrations/voice/trace.py yoyopod/integrations/voice/runtime.py yoyopod/core/bootstrap/screens_boot.py tests/integrations/test_voice_runtime.py
git commit -m "feat: trace voice runtime turns"
```

Expected: quality passes, full pytest passes, commit created.

## Task 5: Final Verification, PR, And Dev Lane Deploy

**Files:**
- Modify only if generated docs drift after prior tasks.

- [ ] **Step 1: Validate dictionary against authored default path**

Run:

```bash
uv run python -m yoyopod_cli.main voice dictionary validate --path data/voice/commands.yaml
```

Expected: exit 0 if the file exists and is valid, or exit 1 with a clear `dictionary file not found` error if the repo does not carry a mutable dictionary yet. Then run the default command and expect exit 0 because the default missing mutable dictionary means built-ins only:

```bash
uv run python -m yoyopod_cli.main voice dictionary validate
```

Expected: prints `OK voice dictionary data/voice/commands.yaml (built-ins only)` when no mutable dictionary file exists.

- [ ] **Step 2: Inspect a local empty trace path**

Run:

```bash
uv run python -m yoyopod_cli.main voice trace last --path .tmp/missing-voice-trace.jsonl
```

Expected: exit 0 and prints `No voice trace entries`.

- [ ] **Step 3: Run required final local gates before push**

Run:

```bash
uv run python scripts/quality.py gate
uv run pytest -q
```

Expected: both pass.

- [ ] **Step 4: Push the branch**

Run:

```bash
git status --short
git push -u origin codex/voice-trace-dictionary-validator
```

Expected: clean or only intentional uncommitted files before push, then branch pushed.

- [ ] **Step 5: Create PR**

Run:

```bash
gh pr create --base main --head codex/voice-trace-dictionary-validator --title "Add voice trace and dictionary validator" --body "## Summary
- add bounded JSONL voice turn trace
- add voice trace CLI viewer
- add voice command dictionary validator
- trace Ask/command outcomes and music focus around voice sessions

## Tests
- uv run python scripts/quality.py gate
- uv run pytest -q"
```

Expected: PR URL printed.

- [ ] **Step 6: Deploy to dev lane on the Raspberry Pi**

Run:

```bash
yoyopod remote mode activate dev
yoyopod remote sync --branch codex/voice-trace-dictionary-validator
```

Expected: dev lane owns hardware and `yoyopod-dev.service` restarts from `/opt/yoyopod-dev/checkout`.

- [ ] **Step 7: Run Pi smoke and inspect trace**

Run:

```bash
yoyopod remote validate --with-cloud-voice
yoyopod remote logs --lines 160 --filter "Voice"
```

Then on the Pi, or through a remote shell command if available, run:

```bash
cd /opt/yoyopod-dev/checkout
/opt/yoyopod-dev/venv/bin/python -m yoyopod_cli.main voice trace last --limit 5
```

Expected: recent command or Ask attempts show source, route kind, outcome, transcript preview, and music focus fields.

- [ ] **Step 8: Manual Pi checks**

Exercise these on the physical device:

```text
1. Start music, hold for voice, say "call mama".
2. Start music, open Ask, ask "why is the sky blue".
3. Start music, hold from hub and ask a general Ask question.
4. Inspect `yoyopod voice trace last --limit 5`.
```

Expected:

```text
- music pauses during capture and resumes after the voice session when no call owns audio
- command route shows command intent for "call mama"
- Ask fallback shows route_kind ask and capped answer preview
- no audio files are created by trace
```
