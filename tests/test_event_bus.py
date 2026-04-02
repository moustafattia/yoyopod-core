"""Tests for the coordinator-thread event bus."""

from __future__ import annotations

import threading
from dataclasses import dataclass

from yoyopy.event_bus import EventBus


@dataclass(frozen=True, slots=True)
class DemoEvent:
    """Simple event used to validate bus behavior."""

    value: str


def test_main_thread_publish_dispatches_inline() -> None:
    """Publishing on the main thread should dispatch immediately."""
    bus = EventBus()
    seen_thread_ids: list[int] = []

    bus.subscribe(DemoEvent, lambda event: seen_thread_ids.append(threading.get_ident()))

    bus.publish(DemoEvent(value="inline"))

    assert seen_thread_ids == [bus.main_thread_id]
    assert bus.drain() == 0


def test_background_publish_is_queued_until_drain() -> None:
    """Background publishes should wait for the coordinator thread to drain them."""
    bus = EventBus()
    seen_thread_ids: list[int] = []

    bus.subscribe(DemoEvent, lambda event: seen_thread_ids.append(threading.get_ident()))

    worker = threading.Thread(target=lambda: bus.publish(DemoEvent(value="queued")))
    worker.start()
    worker.join()

    assert seen_thread_ids == []
    assert bus.drain() == 1
    assert seen_thread_ids == [bus.main_thread_id]
