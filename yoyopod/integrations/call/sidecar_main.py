"""VoIP sidecar process entry point.

Phase 2A ships a scaffold sidecar: it completes the protocol handshake,
echoes :class:`Ping` to :class:`Pong`, and emits a :class:`Log` event
acknowledging each accepted command without performing any liblinphone
work. Phase 2B replaces :func:`_dispatch_command` with the real
:class:`LiblinphoneBackend` integration.

The function :func:`run_sidecar` is what
``yoyopod.integrations.call.sidecar_supervisor.SidecarSupervisor`` passes to
``multiprocessing.Process(target=...)``, so it must remain importable at
module level for both ``spawn`` and ``forkserver`` start methods.
"""

from __future__ import annotations

import sys
from multiprocessing.connection import Connection
from typing import Any

from yoyopod.integrations.call.sidecar_protocol import (
    Accept,
    CallStateChanged,
    Dial,
    Error,
    Hangup,
    Hello,
    Log,
    PROTOCOL_VERSION,
    Ping,
    Pong,
    ProtocolError,
    Ready,
    Register,
    Reject,
    SetMute,
    SetVolume,
    Shutdown,
    Unregister,
    recv_command,
    send_message,
)


def run_sidecar(conn: Connection) -> None:
    """Run the sidecar command loop until :class:`Shutdown` or the pipe closes."""

    _set_parent_death_signal()

    try:
        send_message(conn, Hello(version=PROTOCOL_VERSION, capabilities=("scaffold",)))
    except (BrokenPipeError, EOFError, OSError):
        return

    try:
        peer_hello = recv_command(conn)
    except (BrokenPipeError, EOFError, OSError):
        return
    except ProtocolError as exc:
        _safe_send(
            conn,
            Error(code="protocol_error", message=f"handshake decode failed: {exc}"),
        )
        return

    if not isinstance(peer_hello, Hello) or peer_hello.version != PROTOCOL_VERSION:
        _safe_send(
            conn,
            Error(
                code="protocol_mismatch",
                message=(
                    f"sidecar speaks version {PROTOCOL_VERSION}; "
                    f"received {type(peer_hello).__name__}"
                ),
            ),
        )
        return

    if not _safe_send(conn, Ready()):
        return

    while True:
        try:
            command = recv_command(conn)
        except (BrokenPipeError, EOFError, OSError):
            return
        except ProtocolError as exc:
            if not _safe_send(conn, Error(code="protocol_error", message=str(exc))):
                return
            continue

        if isinstance(command, Shutdown):
            return

        if not _dispatch_command(conn, command):
            return


def _dispatch_command(conn: Connection, command: Any) -> bool:
    """Handle one command in the Phase 2A scaffold mode. Returns False on pipe close."""

    if isinstance(command, Ping):
        return _safe_send(conn, Pong(cmd_id=command.cmd_id))

    if isinstance(command, Register):
        return _safe_send(
            conn,
            Log(
                level="INFO",
                message=f"scaffold sidecar: would register user={command.user!r}",
            ),
        )

    if isinstance(command, Unregister):
        return _safe_send(conn, Log(level="INFO", message="scaffold sidecar: would unregister"))

    if isinstance(command, Dial):
        return _safe_send(
            conn,
            Log(
                level="INFO",
                message=f"scaffold sidecar: would dial uri={command.uri!r}",
            ),
        )

    if isinstance(command, (Accept, Reject, Hangup)):
        action = type(command).__name__.lower()
        return _safe_send(
            conn,
            CallStateChanged(call_id=command.call_id, state=f"scaffold_{action}"),
        )

    if isinstance(command, (SetMute, SetVolume)):
        verb = "mute" if isinstance(command, SetMute) else "volume"
        return _safe_send(
            conn,
            Log(
                level="DEBUG",
                message=(
                    f"scaffold sidecar: ignored {verb} change for call_id={command.call_id!r}"
                ),
            ),
        )

    return _safe_send(
        conn,
        Log(
            level="DEBUG",
            message=f"scaffold sidecar: unhandled command type={type(command).__name__}",
        ),
    )


def _safe_send(conn: Connection, message: Any) -> bool:
    """Send one message, returning False if the parent has closed the pipe."""

    try:
        send_message(conn, message)
        return True
    except (BrokenPipeError, EOFError, OSError):
        return False


def _set_parent_death_signal() -> None:
    """On Linux, ask the kernel to send SIGTERM if the parent process dies."""

    if sys.platform != "linux":
        return

    try:
        import ctypes
        import signal as signal_module

        libc = ctypes.CDLL("libc.so.6", use_errno=True)
        # PR_SET_PDEATHSIG = 1; the second argument is the signal to deliver.
        libc.prctl(1, signal_module.SIGTERM, 0, 0, 0)
    except (OSError, AttributeError):
        # If prctl is unavailable the supervisor still detects death via the
        # closed pipe; this is a defense-in-depth signal, not a hard requirement.
        pass
