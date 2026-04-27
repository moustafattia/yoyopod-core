"""Tests for the yoyopod voice diagnostics CLI."""

from __future__ import annotations

import json
from pathlib import Path

from typer.testing import CliRunner

from yoyopod.integrations.voice.trace import VoiceTraceEntry, VoiceTraceStore
from yoyopod_cli.main import app


runner = CliRunner()


def _entry(turn_id: str, transcript: str) -> VoiceTraceEntry:
    return VoiceTraceEntry(
        turn_id=turn_id,
        started_at="2026-04-27T12:00:00.000Z",
        completed_at="2026-04-27T12:00:01.000Z",
        source="ptt",
        mode="cloud",
        route_kind="command",
        outcome="handled",
        command_intent="ask",
        transcript_raw=transcript,
        transcript_normalized=transcript,
    )


def test_voice_group_is_registered() -> None:
    result = runner.invoke(app, ["voice", "--help"])

    assert result.exit_code == 0
    assert "trace" in result.output


def test_voice_trace_last_prints_recent_rows(tmp_path: Path) -> None:
    trace_path = tmp_path / "turns.jsonl"
    store = VoiceTraceStore(path=trace_path, max_turns=5)
    store.append(_entry("turn-1", "play music"))
    store.append(_entry("turn-2", "why is the sky blue"))

    result = runner.invoke(
        app,
        ["voice", "trace", "last", "--limit", "1", "--path", str(trace_path)],
    )

    assert result.exit_code == 0
    assert "turn-2" in result.output
    assert "why is the sky blue" in result.output
    assert "turn-1" not in result.output


def test_voice_trace_last_tolerates_missing_file(tmp_path: Path) -> None:
    trace_path = tmp_path / "missing.jsonl"

    result = runner.invoke(app, ["voice", "trace", "last", "--path", str(trace_path)])

    assert result.exit_code == 0
    assert "No voice trace entries" in result.output


def test_voice_trace_last_ignores_corrupt_lines(tmp_path: Path) -> None:
    trace_path = tmp_path / "turns.jsonl"
    trace_path.write_text(
        "\n".join(
            [
                "not-json",
                json.dumps(_entry("turn-1", "play music").to_json_dict()),
            ]
        ),
        encoding="utf-8",
    )

    result = runner.invoke(app, ["voice", "trace", "last", "--path", str(trace_path)])

    assert result.exit_code == 0
    assert "turn-1" in result.output
    assert "not-json" not in result.output
