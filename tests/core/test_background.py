"""Tests for the central :class:`BackgroundExecutor` and its named pools."""

from __future__ import annotations

import threading
import time
from concurrent.futures import Future
from typing import Any

import pytest

from yoyopod.core.background import BackgroundExecutor
from yoyopod.core.scheduler import MainThreadScheduler


def _wait_for_pending(scheduler: MainThreadScheduler, *, timeout_seconds: float = 2.0) -> None:
    """Wait until the scheduler has at least one pending main-thread callback."""

    deadline = time.monotonic() + timeout_seconds
    while scheduler.pending_count() == 0 and time.monotonic() < deadline:
        time.sleep(0.005)


def test_submit_and_post_delivers_result_via_scheduler() -> None:
    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler)
    seen: list[str] = []

    future = executor.io.submit_and_post(
        lambda: "ok",
        on_done=lambda fut: seen.append(fut.result()),
    )
    future.result(timeout=2.0)

    _wait_for_pending(scheduler)
    assert scheduler.drain() >= 1
    assert seen == ["ok"]

    executor.shutdown()


def test_submit_and_post_records_background_exception() -> None:
    scheduler = MainThreadScheduler()
    diagnostics: list[dict[str, Any]] = []
    executor = BackgroundExecutor(scheduler, diagnostics_log=diagnostics)

    def boom() -> None:
        raise RuntimeError("kaboom")

    future = executor.io.submit_and_post(boom, on_done=lambda fut: None)

    with pytest.raises(RuntimeError, match="kaboom"):
        future.result(timeout=2.0)

    _wait_for_pending(scheduler)
    scheduler.drain()

    matching = [entry for entry in diagnostics if entry["kind"] == "background_error"]
    assert matching, f"no background_error entry recorded; saw {diagnostics}"
    assert matching[0]["pool"] == "io"
    assert "kaboom" in matching[0]["exc"]

    executor.shutdown()


def test_submit_and_post_records_completion_handler_exception() -> None:
    scheduler = MainThreadScheduler()
    diagnostics: list[dict[str, Any]] = []
    executor = BackgroundExecutor(scheduler, diagnostics_log=diagnostics)

    def handler(_fut: Future[int]) -> None:
        raise ValueError("handler boom")

    future = executor.io.submit_and_post(lambda: 42, on_done=handler)
    future.result(timeout=2.0)

    _wait_for_pending(scheduler)
    scheduler.drain()

    matching = [entry for entry in diagnostics if entry["kind"] == "background_completion_error"]
    assert matching, f"no completion_error entry recorded; saw {diagnostics}"
    assert "handler boom" in matching[0]["exc"]

    executor.shutdown()


def test_submit_returns_future_without_post_callback() -> None:
    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler)

    future = executor.io.submit(lambda: 99)
    assert future.result(timeout=2.0) == 99
    assert scheduler.pending_count() == 0

    executor.shutdown()


def test_register_long_running_records_and_joins_on_shutdown() -> None:
    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler)
    stop_event = threading.Event()

    def loop() -> None:
        while not stop_event.is_set():
            time.sleep(0.01)

    thread = threading.Thread(target=loop, daemon=True, name="poller-test")
    thread.start()
    executor.register_long_running(thread, name="poller-test")

    assert "poller-test" in executor.long_running_thread_names()

    stop_event.set()
    executor.shutdown(timeout=1.0)

    assert not thread.is_alive()


def test_shutdown_returns_when_long_running_thread_misses_timeout() -> None:
    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler)
    release_event = threading.Event()

    def loop() -> None:
        # Wait longer than the shutdown timeout to force the join to time out.
        release_event.wait(timeout=2.0)

    thread = threading.Thread(target=loop, daemon=True, name="stuck-poller")
    thread.start()
    executor.register_long_running(thread, name="stuck-poller")

    started_at = time.monotonic()
    executor.shutdown(timeout=0.1)
    elapsed = time.monotonic() - started_at
    assert elapsed < 1.0  # shutdown should not block for the full thread duration

    release_event.set()
    thread.join(timeout=2.0)


def test_shutdown_completes_within_timeout_when_pool_task_blocks() -> None:
    """Shutdown must honor its timeout even when an in-flight pool task blocks indefinitely."""

    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler, io_workers=1)
    io_started = threading.Event()
    io_release = threading.Event()

    def slow_io() -> None:
        io_started.set()
        # Hold for far longer than the shutdown budget; released in cleanup below.
        io_release.wait(timeout=5.0)

    executor.io.submit(slow_io)
    assert io_started.wait(timeout=1.0), "io task did not start before shutdown"

    started_at = time.monotonic()
    executor.shutdown(timeout=0.1)
    elapsed = time.monotonic() - started_at
    # Without the bounded-shutdown helper, this would block on the running task
    # (cancel_futures only cancels queued work) and elapsed would be ~5s.
    assert elapsed < 1.0, f"shutdown took {elapsed:.2f}s, expected within budget"

    # Cleanup: let the daemon worker exit so it does not linger across tests.
    io_release.set()


def test_shutdown_is_idempotent() -> None:
    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler)
    executor.shutdown()
    executor.shutdown()
    assert executor.is_shutdown() is True


def test_register_after_shutdown_raises() -> None:
    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler)
    executor.shutdown()

    thread = threading.Thread(target=lambda: None, daemon=True)
    with pytest.raises(RuntimeError, match="already shut down"):
        executor.register_long_running(thread, name="late")


def test_set_diagnostics_log_propagates_to_pools() -> None:
    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler)
    diagnostics: list[dict[str, Any]] = []
    executor.set_diagnostics_log(diagnostics)

    def boom() -> None:
        raise RuntimeError("late-attached")

    future = executor.io.submit_and_post(boom, on_done=lambda fut: None)
    with pytest.raises(RuntimeError):
        future.result(timeout=2.0)

    _wait_for_pending(scheduler)
    scheduler.drain()

    assert any("late-attached" in entry["exc"] for entry in diagnostics)

    executor.shutdown()


def test_set_diagnostics_log_propagates_to_watchdog_pool() -> None:
    """Diagnostics sink must reach the dedicated watchdog pool too."""

    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler)
    diagnostics: list[dict[str, Any]] = []
    executor.set_diagnostics_log(diagnostics)

    def boom() -> None:
        raise RuntimeError("watchdog-attached")

    future = executor.watchdog.submit_and_post(boom, on_done=lambda fut: None)
    with pytest.raises(RuntimeError):
        future.result(timeout=2.0)

    _wait_for_pending(scheduler)
    scheduler.drain()

    matching = [
        entry
        for entry in diagnostics
        if entry["pool"] == "watchdog" and "watchdog-attached" in entry["exc"]
    ]
    assert matching, f"no watchdog-pool error recorded; saw {diagnostics}"

    executor.shutdown()


def test_subprocess_pool_isolated_from_io_pool() -> None:
    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler, io_workers=1, subprocess_workers=1)
    io_started = threading.Event()
    io_release = threading.Event()

    def slow_io() -> None:
        io_started.set()
        io_release.wait(timeout=2.0)

    # Saturate the io pool with a long-blocked task.
    blocker = executor.io.submit(slow_io)
    assert io_started.wait(timeout=1.0)

    # Subprocess pool should still process work in parallel.
    sub_done = executor.subprocess.submit(lambda: "sub-ok")
    assert sub_done.result(timeout=1.0) == "sub-ok"

    io_release.set()
    blocker.result(timeout=2.0)
    executor.shutdown()


def test_watchdog_pool_isolated_from_io_pool() -> None:
    """Watchdog feeds must run on workers that cloud/io work cannot saturate."""

    scheduler = MainThreadScheduler()
    executor = BackgroundExecutor(scheduler, io_workers=1)
    io_started = threading.Event()
    io_release = threading.Event()

    def slow_io() -> None:
        io_started.set()
        io_release.wait(timeout=2.0)

    # Saturate the io pool with a long-blocked task that mirrors a stuck
    # CloudManager HTTP call.
    blocker = executor.io.submit(slow_io)
    assert io_started.wait(timeout=1.0)

    # The watchdog pool must run independently and complete promptly.
    wd_done = executor.watchdog.submit(lambda: "wd-ok")
    assert wd_done.result(timeout=1.0) == "wd-ok"

    io_release.set()
    blocker.result(timeout=2.0)
    executor.shutdown()
