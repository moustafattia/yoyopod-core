# Voice Trace And Dictionary Validator Design

Date: 2026-04-28
Status: Draft for review
Branch: codex/voice-trace-dictionary-validator

## Context

PR #390 merged the cautious unified voice work. The next weakness is observability:
when a real device says "no such voice command" or silently routes through Ask,
we need enough structured evidence to see what the device heard, how it matched,
what it tried to do, and what audio focus did around the session.

This spec covers a bounded rolling JSONL voice trace plus a command dictionary
validator. It intentionally does not add new voice behavior.

## Goals

- Capture a compact per-turn trace for command and Ask sessions on the Pi.
- Enable tracing in dev and prod, with different retention limits.
- Avoid audio artifacts and full Ask history storage.
- Add a CLI path to inspect recent voice turns without SSH log archaeology.
- Add a dictionary validator that catches broken examples, aliases, and actions
  before deploy.
- Keep diagnostics out of the on-device UI for this PR.

## Non-Goals

- No wake-word implementation.
- No audio file recording or retention.
- No full Ask transcript history beyond the bounded per-turn trace fields.
- No command matching behavior changes except test fixtures needed by the
  validator.
- No cloud upload, analytics pipeline, or remote dashboard.
- No new UI diagnostics screen.

## Configuration

Add `VoiceTraceConfig` under `VoiceConfig` as `voice.trace`. It should be a peer
to the existing voice runtime settings, not a property of a specific screen.

Fields:

- `enabled: bool = true`
- `path: str = "logs/voice/turns.jsonl"`
- `max_turns: int = 50`
- `include_transcripts: bool = true`
- `body_preview_chars: int = 160`

Environment overrides:

- `YOYOPOD_VOICE_TRACE_ENABLED`
- `YOYOPOD_VOICE_TRACE_PATH`
- `YOYOPOD_VOICE_TRACE_MAX_TURNS`
- `YOYOPOD_VOICE_TRACE_INCLUDE_TRANSCRIPTS`
- `YOYOPOD_VOICE_TRACE_BODY_PREVIEW_CHARS`

The authored safe default is 50 turns, suitable for prod. Dev lane config should
override `max_turns` to 200 if the existing lane configuration path supports a
clean override. If the lane config path cannot express this cleanly, use the
environment override in the dev service configuration and keep the
production-safe default in code.

## Trace Store

Add a small trace module at `yoyopod/integrations/voice/trace.py` with two
responsibilities:

- Convert a voice turn into a JSON-serializable entry.
- Append and rotate a bounded JSONL file without interrupting voice behavior.

The store should append one JSON object per completed voice turn. After append,
it should retain only the latest `max_turns` valid entries and rewrite the file
atomically. Corrupt or partial lines should be ignored during rotation and CLI
reads, with a debug log at most. Trace write failures must never fail a voice
session.

Recommended objects:

- `VoiceTraceEntry`
- `VoiceTraceTiming`
- `VoiceTraceAudioFocus`
- `VoiceTraceMusicState`
- `VoiceTraceStore`

## Trace Schema

Use `schema_version: 1` and keep fields stable enough for scripts. Text fields
must be capped before writing.

Required fields:

- `schema_version`
- `turn_id`
- `started_at`
- `completed_at`
- `source`: `ask_screen`, `hub_hold`, `ptt`, `voice_command_event`, or `unknown`
- `mode`: `ask`, `command`, `ptt`, or `unknown`
- `route_kind`: `command`, `ask`, `silence`, `error`, or `unknown`
- `outcome`: short machine-readable summary
- `error`: null or `{stage, type, message}`

Conditional or optional fields:

- `transcript_raw`
- `transcript_normalized`
- `activation_prefix`
- `command_intent`
- `command_confidence`
- `route_name`
- `ask_fallback`
- `assistant_status`
- `assistant_title`
- `assistant_body_preview`
- `should_speak`
- `auto_return`
- `timings_ms`
- `audio_focus_before`
- `audio_focus_after`
- `music_before`
- `music_after`

Privacy and size rules:

- Do not store audio paths or audio bytes.
- Do not store complete Ask answers; only `assistant_body_preview`, capped by
  `voice.trace.body_preview_chars`.
- If `include_transcripts` is false, omit raw and normalized transcript fields.
- Cap transcript strings as part of the trace writer, not only at call sites.

Example entry:

```json
{"schema_version":1,"turn_id":"01HX...","source":"ask_screen","mode":"command","route_kind":"command","transcript_normalized":"call mama","command_intent":"call_contact","outcome":"command_started","timings_ms":{"total":812},"audio_focus_before":{"music_state":"playing"},"audio_focus_after":{"music_state":"resumed"},"error":null}
```

## Runtime Integration

The trace should be captured from `VoiceRuntimeCoordinator`, but the coordinator
should not become a dumping ground for JSON details. Use a narrow recorder object
that receives lifecycle updates:

- Session started: source, mode, screen, audio focus before, music state before.
- STT completed or failed: transcript, language if available, timing, error.
- Router completed: route kind, command intent, confidence, Ask fallback.
- Action completed: command result or Ask response metadata.
- Audio focus completed: final music state and whether playback was resumed.
- Session completed: final outcome and total duration.

Both the Ask screen and hub hold path should use the same runtime route after
the previous unified voice work, so the trace should expose any remaining source
differences rather than create separate implementations.

Trace failures should be swallowed after a debug log. Voice, music, VoIP, and UI
state must be unaffected by trace storage.

## CLI

Add a new Typer subapp at `yoyopod_cli/voice.py`, registered from
`yoyopod_cli/main.py` as `yoyopod voice`.

Commands:

- `yoyopod voice trace last --limit 5`
- `yoyopod voice trace last --limit 20 --path logs/voice/turns.jsonl`
- `yoyopod voice dictionary validate`
- `yoyopod voice dictionary validate --path data/voice/commands.yaml`
- `yoyopod voice dictionary validate --strict`

`trace last` should print compact human-readable rows with newest entries first.
JSON output is intentionally not required in the first PR.

`dictionary validate` should return exit code 0 when there are no errors and
exit code 1 when there are errors. Warnings should not fail unless `--strict` is
set.

## Dictionary Validator

Add a validator module at `yoyopod/integrations/voice/dictionary_validator.py`.
It should use the same loading and matching functions as runtime command
matching where possible.

Validation errors:

- YAML cannot be parsed.
- Required command fields are missing or have the wrong type.
- A route action is not in the safe route action set.
- An example fails to match its owning command intent.
- Two commands claim the same alias for different intents.
- A dictionary entry cannot be loaded into the runtime matcher.

Validation warnings:

- Alias or example text is unusually short and may be ambiguous.
- An example differs only by punctuation or spacing from another example.
- A contact-style example appears to reference a missing contact, if a people
  directory is available to the validator.

The validator should print file and command identifiers for every issue. It
should be deterministic so failures are easy to compare in CI and deploy logs.

## Testing

Unit tests:

- Trace config defaults and env overrides.
- Trace entry serialization and text capping.
- JSONL append and rotation at `max_turns`.
- Corrupt line tolerance during rotation and reads.
- CLI formatting for `voice trace last`.
- Dictionary validator success, error, warning, and strict-warning cases.

Runtime tests:

- Command turn creates a trace with transcript, intent, route, and outcome.
- Ask fallback creates a trace with Ask metadata and capped body preview.
- STT failure creates an error trace without raising.
- Music playing before voice is represented before and after audio focus.

Quality gates:

- `uv run python scripts/quality.py gate`
- `uv run pytest -q`

## Rollout

1. Land this spec.
2. After review, write the implementation plan.
3. Implement trace storage, config, CLI, validator, and tests in one focused PR.
4. Deploy to the dev lane and reproduce at least:
   - "call mama" command success or validator failure.
   - Ask fallback from the Ask screen.
   - Ask or command from hub hold.
   - Music pause and resume around a voice session.
5. Inspect the rolling trace on the Pi with `yoyopod voice trace last`.

## Resolved Decisions

- Use rolling JSONL at `logs/voice/turns.jsonl`.
- Enable tracing in both dev and prod.
- Keep 200 turns in dev and 50 turns in prod where lane config supports it.
- Store bounded transcript text by default.
- Store no audio files and no full Ask history.
