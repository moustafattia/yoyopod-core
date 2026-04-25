"""Integration tests for ``app.background`` wired through :class:`YoyoPodApp`."""

from __future__ import annotations

import threading
import time
from concurrent.futures import Future
from typing import Any

from yoyopod.core import YoyoPodApp


def _wait_for_pending(app: YoyoPodApp, *, timeout_seconds: float = 2.0) -> None:
    """Wait until the scheduler has at least one pending main-thread callback."""

    deadline = time.monotonic() + timeout_seconds
    while app.scheduler.pending_count() == 0 and time.monotonic() < deadline:
        time.sleep(0.005)


def test_app_background_delivers_results_to_scheduler() -> None:
    app = YoyoPodApp(strict_bus=True)
    app.start()
    seen: list[int] = []

    future = app.background.io.submit_and_post(
        lambda: 21 * 2,
        on_done=lambda fut: seen.append(fut.result()),
    )
    future.result(timeout=2.0)
    _wait_for_pending(app)
    app.scheduler.drain()

    assert seen == [42]
    app.stop()


def test_app_background_records_failures_on_log_buffer() -> None:
    app = YoyoPodApp(strict_bus=True)
    app.start()

    def boom() -> None:
        raise RuntimeError("background-boom")

    future = app.background.io.submit_and_post(boom, on_done=lambda fut: None)
    try:
        future.result(timeout=2.0)
    except RuntimeError:
        pass
    _wait_for_pending(app)
    app.scheduler.drain()

    entries: list[Any] = app.log_buffer.snapshot()
    matches = [
        entry
        for entry in entries
        if isinstance(entry, dict) and entry.get("kind") == "background_error"
    ]
    assert matches, f"no background_error entry found; saw {entries!r}"
    assert "background-boom" in matches[0]["exc"]

    app.stop()


def test_app_stop_shuts_down_background_executor() -> None:
    app = YoyoPodApp(strict_bus=True)
    app.start()

    assert app.background.is_shutdown() is False
    app.stop()
    assert app.background.is_shutdown() is True


def test_app_background_joins_registered_long_running_thread() -> None:
    app = YoyoPodApp(strict_bus=True)
    app.start()
    stop_event = threading.Event()

    def loop() -> None:
        while not stop_event.is_set():
            time.sleep(0.01)

    poller_thread = threading.Thread(target=loop, daemon=True, name="integration-poller")
    poller_thread.start()
    app.background.register_long_running(poller_thread, name="integration-poller")
    assert "integration-poller" in app.background.long_running_thread_names()

    stop_event.set()
    app.stop()

    assert not poller_thread.is_alive()


def test_app_background_subprocess_pool_isolated_from_io_pool() -> None:
    app = YoyoPodApp(strict_bus=True)
    app.start()

    # Use both pools; check that the pool attribute objects are distinct.
    io_future: Future[str] = app.background.io.submit(lambda: "io")
    sub_future: Future[str] = app.background.subprocess.submit(lambda: "sub")

    assert io_future.result(timeout=2.0) == "io"
    assert sub_future.result(timeout=2.0) == "sub"
    assert app.background.io is not app.background.subprocess

    app.stop()
