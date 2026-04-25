"""Central background-work executor for off-thread I/O and subprocess operations.

Two named `ThreadPoolExecutor` pools (`io`, `subprocess`) run blocking work
off the coordinator thread, and a registry of long-running poller threads
provides centralized shutdown discipline. All Future results return to the
coordinator via :meth:`MainThreadScheduler.post` so call sites never touch
threading primitives directly.
"""

from __future__ import annotations

import threading
from concurrent.futures import Future, ThreadPoolExecutor
from dataclasses import dataclass
from typing import Any, Callable, Protocol, TypeVar

from loguru import logger


class _Scheduler(Protocol):
    def post(self, fn: Callable[[], None]) -> None:
        """Queue a callback for main-thread drain."""


class _DiagnosticsLog(Protocol):
    def append(self, entry: Any) -> None:
        """Append one diagnostics entry."""


_T = TypeVar("_T")
_DEFAULT_IO_WORKERS = 4
_DEFAULT_SUBPROCESS_WORKERS = 2
_DEFAULT_SHUTDOWN_TIMEOUT_SECONDS = 5.0


@dataclass(slots=True)
class _LongRunningEntry:
    """One registered long-running thread joined by :meth:`BackgroundExecutor.shutdown`."""

    name: str
    thread: threading.Thread


class BackgroundPool:
    """Wrap one `ThreadPoolExecutor` with a coordinator-thread result-delivery helper."""

    def __init__(
        self,
        *,
        executor: ThreadPoolExecutor,
        scheduler: _Scheduler,
        diagnostics_log: _DiagnosticsLog | None,
        pool_name: str,
    ) -> None:
        self._executor = executor
        self._scheduler = scheduler
        self._diagnostics_log = diagnostics_log
        self._pool_name = pool_name

    def submit(self, fn: Callable[..., _T], *args: Any, **kwargs: Any) -> Future[_T]:
        """Submit work to the pool; the caller owns the returned `Future`."""

        return self._executor.submit(fn, *args, **kwargs)

    def submit_and_post(
        self,
        fn: Callable[..., _T],
        *args: Any,
        on_done: Callable[[Future[_T]], None],
        **kwargs: Any,
    ) -> Future[_T]:
        """Submit work and deliver the completed `Future` back via the main-thread scheduler."""

        future = self._executor.submit(fn, *args, **kwargs)
        future.add_done_callback(self._build_post_callback(fn, on_done))
        return future

    def _set_diagnostics_log(self, diagnostics_log: _DiagnosticsLog | None) -> None:
        """Replace the diagnostics sink for this pool."""

        self._diagnostics_log = diagnostics_log

    def _build_post_callback(
        self,
        fn: Callable[..., Any],
        on_done: Callable[[Future[Any]], None],
    ) -> Callable[[Future[Any]], None]:
        """Return a Future done-callback that posts the handler onto the main thread."""

        def _post(future: Future[Any]) -> None:
            self._scheduler.post(lambda: self._invoke_handler(fn, future, on_done))

        return _post

    def _invoke_handler(
        self,
        fn: Callable[..., Any],
        future: Future[Any],
        on_done: Callable[[Future[Any]], None],
    ) -> None:
        """Run the completion handler on the main thread, recording any failures."""

        background_exc = future.exception()
        if background_exc is not None:
            self._record_error(
                kind="background_error",
                fn=fn,
                exc=background_exc,
            )
            logger.opt(exception=background_exc).error(
                "Background work failed in pool {!r}: {}",
                self._pool_name,
                _callable_name(fn),
            )
        try:
            on_done(future)
        except Exception as exc:
            self._record_error(
                kind="background_completion_error",
                fn=fn,
                exc=exc,
            )
            logger.exception(
                "Background completion handler for {} failed",
                _callable_name(fn),
            )

    def _record_error(
        self,
        *,
        kind: str,
        fn: Callable[..., Any],
        exc: BaseException,
    ) -> None:
        """Append one background-error entry to the diagnostics sink when present."""

        if self._diagnostics_log is None:
            return
        self._diagnostics_log.append(
            {
                "kind": kind,
                "pool": self._pool_name,
                "handler": _callable_name(fn),
                "exc": f"{exc.__class__.__name__}: {exc}",
            }
        )


class BackgroundExecutor:
    """Owns named background pools and registered long-running poller threads."""

    def __init__(
        self,
        scheduler: _Scheduler,
        *,
        diagnostics_log: _DiagnosticsLog | None = None,
        io_workers: int = _DEFAULT_IO_WORKERS,
        subprocess_workers: int = _DEFAULT_SUBPROCESS_WORKERS,
    ) -> None:
        self._scheduler = scheduler
        self._diagnostics_log = diagnostics_log
        self._io_executor = ThreadPoolExecutor(max_workers=io_workers, thread_name_prefix="yp-io")
        self._subprocess_executor = ThreadPoolExecutor(
            max_workers=subprocess_workers, thread_name_prefix="yp-sub"
        )
        self.io = BackgroundPool(
            executor=self._io_executor,
            scheduler=scheduler,
            diagnostics_log=diagnostics_log,
            pool_name="io",
        )
        self.subprocess = BackgroundPool(
            executor=self._subprocess_executor,
            scheduler=scheduler,
            diagnostics_log=diagnostics_log,
            pool_name="subprocess",
        )
        self._long_running: list[_LongRunningEntry] = []
        self._shutdown = False
        self._shutdown_lock = threading.Lock()

    def register_long_running(
        self,
        thread: threading.Thread,
        *,
        name: str,
    ) -> None:
        """Register a poller thread for centralized shutdown discipline."""

        with self._shutdown_lock:
            if self._shutdown:
                raise RuntimeError(
                    f"Cannot register long-running thread {name!r}: "
                    "BackgroundExecutor already shut down"
                )
            self._long_running.append(_LongRunningEntry(name=name, thread=thread))

    def set_diagnostics_log(self, diagnostics_log: _DiagnosticsLog | None) -> None:
        """Attach or clear the diagnostics sink and propagate to managed pools."""

        self._diagnostics_log = diagnostics_log
        self.io._set_diagnostics_log(diagnostics_log)
        self.subprocess._set_diagnostics_log(diagnostics_log)

    def long_running_thread_names(self) -> list[str]:
        """Return the names of currently registered long-running threads."""

        with self._shutdown_lock:
            return [entry.name for entry in self._long_running]

    def is_shutdown(self) -> bool:
        """Return whether :meth:`shutdown` has already been called."""

        with self._shutdown_lock:
            return self._shutdown

    def shutdown(self, *, timeout: float = _DEFAULT_SHUTDOWN_TIMEOUT_SECONDS) -> None:
        """Cancel pending Futures, join workers, and join registered long-running threads."""

        with self._shutdown_lock:
            if self._shutdown:
                return
            self._shutdown = True
            entries = list(self._long_running)

        self._io_executor.shutdown(wait=True, cancel_futures=True)
        self._subprocess_executor.shutdown(wait=True, cancel_futures=True)
        for entry in entries:
            entry.thread.join(timeout=timeout)
            if entry.thread.is_alive():
                logger.warning(
                    "Long-running background thread {!r} did not exit within {}s",
                    entry.name,
                    timeout,
                )


def _callable_name(fn: Callable[..., Any]) -> str:
    """Return a stable identifier for a callable used in diagnostics entries."""

    module = getattr(fn, "__module__", "") or ""
    qualname = getattr(fn, "__qualname__", getattr(fn, "__name__", repr(fn)))
    return f"{module}.{qualname}".strip(".")
