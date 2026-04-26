from __future__ import annotations

import threading
from pathlib import Path
from collections.abc import Callable

import pytest

from yoyopod.core.events import WorkerMessageReceivedEvent
from yoyopod.integrations.voice.worker_client import VoiceWorkerClient, VoiceWorkerTimeout
from yoyopod.integrations.voice.worker_contract import (
    VoiceWorkerSpeakResult,
    VoiceWorkerTranscribeResult,
)


class _Scheduler:
    def __init__(self) -> None:
        self.callbacks: list[Callable[[], None]] = []

    def run_on_main(self, callback: Callable[[], None]) -> None:
        self.callbacks.append(callback)

    def drain(self) -> None:
        callbacks = list(self.callbacks)
        self.callbacks.clear()
        for callback in callbacks:
            callback()


class _Supervisor:
    def __init__(self) -> None:
        self.requests: list[dict[str, object]] = []
        self.send_result = True

    def send_request(self, domain: str, **kwargs: object) -> bool:
        self.requests.append({"domain": domain, **kwargs})
        return self.send_result


def test_transcribe_schedules_request_on_main_and_resolves_result() -> None:
    scheduler = _Scheduler()
    supervisor = _Supervisor()
    client = VoiceWorkerClient(
        scheduler=scheduler,
        worker_supervisor=supervisor,
        request_timeout_seconds=0.25,
    )
    results: list[VoiceWorkerTranscribeResult] = []

    thread = threading.Thread(
        target=lambda: results.append(
            client.transcribe(
                audio_path=Path("/tmp/input.wav"),
                sample_rate_hz=16000,
                language="en",
                max_audio_seconds=5.0,
            )
        )
    )
    thread.start()

    _wait_until(lambda: len(scheduler.callbacks) == 1)
    assert supervisor.requests == []
    scheduler.drain()

    assert len(supervisor.requests) == 1
    request = supervisor.requests[0]
    assert request["domain"] == "voice"
    assert request["type"] == "voice.transcribe"
    assert isinstance(request["request_id"], str)
    assert str(request["request_id"]).startswith("voice-")
    assert request["timeout_seconds"] == 0.25
    assert request["payload"] == {
        "audio_path": "/tmp/input.wav",
        "format": "wav",
        "sample_rate_hz": 16000,
        "channels": 1,
        "language": "en",
        "max_audio_seconds": 5.0,
        "delete_input_on_success": False,
    }

    client.handle_worker_message(
        WorkerMessageReceivedEvent(
            domain="voice",
            kind="result",
            type="voice.transcribe.result",
            request_id=str(request["request_id"]),
            payload={"text": " play music ", "confidence": 0.9, "is_final": True},
        )
    )
    thread.join(timeout=1.0)

    assert not thread.is_alive()
    assert results == [
        VoiceWorkerTranscribeResult(text="play music", confidence=0.9, is_final=True)
    ]
    assert client.pending_count == 0


def test_worker_error_raises_runtime_error_with_error_code() -> None:
    scheduler = _Scheduler()
    supervisor = _Supervisor()
    client = VoiceWorkerClient(
        scheduler=scheduler,
        worker_supervisor=supervisor,
        request_timeout_seconds=0.25,
    )
    errors: list[BaseException] = []

    thread = threading.Thread(
        target=lambda: _capture_error(
            errors,
            lambda: client.transcribe(
                audio_path=Path("/tmp/input.wav"),
                sample_rate_hz=16000,
                language="en",
                max_audio_seconds=5.0,
            ),
        )
    )
    thread.start()

    _wait_until(lambda: len(scheduler.callbacks) == 1)
    scheduler.drain()
    request_id = str(supervisor.requests[0]["request_id"])

    client.handle_worker_message(
        WorkerMessageReceivedEvent(
            domain="voice",
            kind="error",
            type="voice.error",
            request_id=request_id,
            payload={
                "code": "provider_unavailable",
                "message": "provider down",
                "retryable": True,
            },
        )
    )
    thread.join(timeout=1.0)

    assert not thread.is_alive()
    assert len(errors) == 1
    assert isinstance(errors[0], RuntimeError)
    assert "provider_unavailable" in str(errors[0])
    assert client.pending_count == 0


def test_transcribe_raises_timeout_when_no_result_arrives() -> None:
    scheduler = _Scheduler()
    supervisor = _Supervisor()
    client = VoiceWorkerClient(
        scheduler=scheduler,
        worker_supervisor=supervisor,
        request_timeout_seconds=0.01,
    )

    with pytest.raises(VoiceWorkerTimeout):
        client.transcribe(
            audio_path=Path("/tmp/input.wav"),
            sample_rate_hz=16000,
            language="en",
            max_audio_seconds=5.0,
        )

    assert scheduler.callbacks
    assert supervisor.requests == []
    assert client.pending_count == 0


def test_messages_from_other_domain_and_unknown_request_are_ignored() -> None:
    scheduler = _Scheduler()
    supervisor = _Supervisor()
    client = VoiceWorkerClient(
        scheduler=scheduler,
        worker_supervisor=supervisor,
        request_timeout_seconds=0.25,
    )
    results: list[VoiceWorkerTranscribeResult] = []

    thread = threading.Thread(
        target=lambda: results.append(
            client.transcribe(
                audio_path=Path("/tmp/input.wav"),
                sample_rate_hz=16000,
                language="en",
                max_audio_seconds=5.0,
            )
        )
    )
    thread.start()

    _wait_until(lambda: len(scheduler.callbacks) == 1)
    scheduler.drain()
    request_id = str(supervisor.requests[0]["request_id"])

    client.handle_worker_message(
        WorkerMessageReceivedEvent(
            domain="music",
            kind="result",
            type="voice.transcribe.result",
            request_id=request_id,
            payload={"text": "wrong", "confidence": 1.0, "is_final": True},
        )
    )
    client.handle_worker_message(
        WorkerMessageReceivedEvent(
            domain="voice",
            kind="result",
            type="voice.transcribe.result",
            request_id="voice-unknown",
            payload={"text": "wrong", "confidence": 1.0, "is_final": True},
        )
    )
    client.handle_worker_message(
        WorkerMessageReceivedEvent(
            domain="voice",
            kind="event",
            type="voice.progress",
            request_id=request_id,
            payload={"percent": 50},
        )
    )

    assert results == []
    assert client.pending_count == 1

    client.handle_worker_message(
        WorkerMessageReceivedEvent(
            domain="voice",
            kind="result",
            type="voice.transcribe.result",
            request_id=request_id,
            payload={"text": "correct", "confidence": 0.8, "is_final": True},
        )
    )
    thread.join(timeout=1.0)

    assert not thread.is_alive()
    assert results == [VoiceWorkerTranscribeResult(text="correct", confidence=0.8, is_final=True)]
    assert client.pending_count == 0


def test_same_request_unrelated_error_kind_message_is_ignored() -> None:
    scheduler = _Scheduler()
    supervisor = _Supervisor()
    client = VoiceWorkerClient(
        scheduler=scheduler,
        worker_supervisor=supervisor,
        request_timeout_seconds=0.25,
    )
    results: list[VoiceWorkerTranscribeResult] = []
    errors: list[BaseException] = []

    thread = threading.Thread(
        target=lambda: _capture_error(
            errors,
            lambda: results.append(
                client.transcribe(
                    audio_path=Path("/tmp/input.wav"),
                    sample_rate_hz=16000,
                    language="en",
                    max_audio_seconds=5.0,
                )
            ),
        )
    )
    thread.start()

    _wait_until(lambda: len(scheduler.callbacks) == 1)
    scheduler.drain()
    request_id = str(supervisor.requests[0]["request_id"])

    client.handle_worker_message(
        WorkerMessageReceivedEvent(
            domain="voice",
            kind="error",
            type="voice.progress",
            request_id=request_id,
            payload={
                "code": "progress_parse_failure",
                "message": "progress event failed upstream",
            },
        )
    )

    assert results == []
    assert errors == []
    assert client.pending_count == 1
    assert thread.is_alive()

    client.handle_worker_message(
        WorkerMessageReceivedEvent(
            domain="voice",
            kind="result",
            type="voice.transcribe.result",
            request_id=request_id,
            payload={"text": "correct", "confidence": 0.8, "is_final": True},
        )
    )
    thread.join(timeout=1.0)

    assert not thread.is_alive()
    assert errors == []
    assert results == [VoiceWorkerTranscribeResult(text="correct", confidence=0.8, is_final=True)]
    assert client.pending_count == 0


def test_speak_schedules_request_on_main_and_resolves_result() -> None:
    scheduler = _Scheduler()
    supervisor = _Supervisor()
    client = VoiceWorkerClient(
        scheduler=scheduler,
        worker_supervisor=supervisor,
        request_timeout_seconds=0.25,
    )
    results: list[VoiceWorkerSpeakResult] = []

    thread = threading.Thread(
        target=lambda: results.append(
            client.speak(
                text="Playing music",
                voice="alloy",
                model="gpt-4o-mini-tts",
                instructions="Speak clearly.",
                sample_rate_hz=16000,
            )
        )
    )
    thread.start()

    _wait_until(lambda: len(scheduler.callbacks) == 1)
    scheduler.drain()
    request = supervisor.requests[0]

    assert request["domain"] == "voice"
    assert request["type"] == "voice.speak"
    assert request["payload"] == {
        "text": "Playing music",
        "voice": "alloy",
        "model": "gpt-4o-mini-tts",
        "instructions": "Speak clearly.",
        "format": "wav",
        "sample_rate_hz": 16000,
    }

    client.handle_worker_message(
        WorkerMessageReceivedEvent(
            domain="voice",
            kind="result",
            type="voice.speak.result",
            request_id=str(request["request_id"]),
            payload={
                "audio_path": "/tmp/output.wav",
                "format": "wav",
                "sample_rate_hz": 16000,
                "duration_ms": 830,
            },
        )
    )
    thread.join(timeout=1.0)

    assert not thread.is_alive()
    assert results == [
        VoiceWorkerSpeakResult(
            audio_path=Path("/tmp/output.wav"),
            format="wav",
            sample_rate_hz=16000,
            duration_ms=830,
        )
    ]
    assert client.pending_count == 0


def _capture_error(errors: list[BaseException], callback: Callable[[], object]) -> None:
    try:
        callback()
    except BaseException as exc:
        errors.append(exc)


def _wait_until(predicate: Callable[[], bool], timeout_seconds: float = 1.0) -> None:
    deadline = threading.Event()
    completed = threading.Event()

    def poll() -> None:
        while not completed.is_set() and not predicate():
            completed.wait(0.001)
        deadline.set()

    thread = threading.Thread(target=poll)
    thread.start()
    assert deadline.wait(timeout_seconds)
    completed.set()
    thread.join(timeout=1.0)
