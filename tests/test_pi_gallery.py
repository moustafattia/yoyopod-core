"""Tests for gallery CLI support modules."""

from __future__ import annotations

import pytest

from yoyopod.cli.pi.gallery.capture import _build_capture_specs
from yoyopod.cli.pi.gallery.command import (
    _ASK_RESPONSE_BODY,
    _ASK_RESPONSE_HEADLINE,
    _advance_ask_to_response,
)
from yoyopod.cli.pi.gallery.fixtures import (
    _GALLERY_POWER_STATUS_FIELDS,
    _build_call_history_store,
    _build_contacts,
    _build_music_service,
    _build_power_snapshot,
    _build_power_status,
)


class _StubAskScreen:
    """Minimal AskScreen-like object for response-state tests."""

    def __init__(self) -> None:
        self.response: tuple[str, str] | None = None

    def _set_response(self, headline: str, body: str) -> None:
        self.response = (headline, body)


def test_gallery_fixture_builders_return_deterministic_data() -> None:
    """The gallery fixture builders should expose stable demo data."""

    contacts = _build_contacts()
    assert [contact.name for contact in contacts] == ["Hagar", "Mama", "Papa", "Auntie"]

    music_service = _build_music_service()
    assert [playlist.name for playlist in music_service.list_playlists()] == [
        "Morning Boost",
        "Arabic Favorites",
        "Wind Down",
    ]
    assert [track.title for track in music_service.list_recent_tracks()] == [
        "Golden Hour",
        "Midnight Train",
        "Coastline",
    ]

    call_history_store = _build_call_history_store()
    assert call_history_store.missed_count() == 1
    assert call_history_store.recent_preview() == ["Mama", "Papa", "Auntie"]

    power_snapshot = _build_power_snapshot()
    assert power_snapshot.available is True
    assert power_snapshot.device.model == "PiSugar 3"


def test_gallery_power_status_uses_declared_schema() -> None:
    """The gallery power payload should stay aligned with its declared fixture schema."""

    status = _build_power_status()

    assert tuple(status) == _GALLERY_POWER_STATUS_FIELDS
    assert status["shutdown_in_seconds"] is None
    assert status["watchdog_active"] is True


def test_advance_ask_to_response_sets_reply_state_directly() -> None:
    """The Ask response helper should use the screen response setter directly."""

    screen = _StubAskScreen()

    _advance_ask_to_response(screen)

    assert screen.response == (_ASK_RESPONSE_HEADLINE, _ASK_RESPONSE_BODY)


def test_advance_ask_to_response_requires_response_setter() -> None:
    """The Ask response helper should fail fast when passed the wrong screen type."""

    with pytest.raises(TypeError, match="_set_response"):
        _advance_ask_to_response(object())


def test_build_capture_specs_wires_ask_response_prepare_callback() -> None:
    """The Ask response capture should use the supplied prepare callback."""

    prepared: list[object] = []
    specs = _build_capture_specs(
        object(),
        advance_ask_to_response=lambda screen: prepared.append(screen),
    )
    ask_response_spec = next(spec for spec in specs if spec.name == "11_ask_response")
    screen = object()

    assert ask_response_spec.prepare is not None
    ask_response_spec.prepare(screen)

    assert prepared == [screen]
