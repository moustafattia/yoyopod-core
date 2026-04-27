"""Voice diagnostics CLI commands."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import typer

from yoyopod.integrations.voice.trace import DEFAULT_VOICE_TRACE_PATH, VoiceTraceStore

app = typer.Typer(name="voice", help="Voice diagnostics and validation.", no_args_is_help=True)
trace_app = typer.Typer(
    name="trace", help="Inspect local voice trace entries.", no_args_is_help=True
)
app.add_typer(trace_app, name="trace")


def _cell(value: Any, width: int = 28) -> str:
    text = " ".join(str(value or "").split())
    if len(text) > width:
        if width <= 3:
            text = text[:width]
        else:
            text = f"{text[: width - 3]}..."
    return text.ljust(width)


def _row(payload: dict[str, Any]) -> str:
    target = payload.get("command_intent") or payload.get("route_name") or ""
    text = payload.get("transcript_normalized") or payload.get("assistant_body_preview") or ""
    values = (
        payload.get("completed_at", ""),
        payload.get("turn_id", ""),
        payload.get("source", ""),
        payload.get("mode", ""),
        payload.get("route_kind", ""),
        payload.get("outcome", ""),
        target,
        text,
    )
    return " ".join(_cell(value) for value in values).rstrip()


@trace_app.command(name="last")
def trace_last(
    limit: int = typer.Option(5, "--limit", min=1, help="Number of recent trace entries to show."),
    path: Path = typer.Option(
        DEFAULT_VOICE_TRACE_PATH,
        "--path",
        help="Path to the voice trace JSONL file.",
    ),
) -> None:
    """Show the most recent local voice trace entries."""

    rows = VoiceTraceStore(path=path, max_turns=max(1, limit)).read_recent(limit=limit)
    rows = list(reversed(rows))
    if not rows:
        typer.echo(f"No voice trace entries at {path}")
        return

    typer.echo(
        " ".join(
            _cell(value)
            for value in (
                "completed_at",
                "turn_id",
                "source",
                "mode",
                "route_kind",
                "outcome",
                "target",
                "text",
            )
        ).rstrip()
    )
    for payload in rows:
        typer.echo(_row(payload))
