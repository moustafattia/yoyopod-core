# Phase A — Plan 3: Easy Batch (Network, Location, Contacts, Cloud) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Migrate four data-oriented integrations that share the same shape as the power pilot and don't cross-cut other domains: `network`, `location` (split out of `network/gps.py`), `contacts`, and `cloud`. With the pilot's template now proven, these four fall in sequence using the same backend-move + integration-setup pattern.

**Architecture:** Each integration follows the Plan-2 template — move adapter under `src/yoyopod/backends/<name>/`, create `src/yoyopod/integrations/<name>/` with typed commands, handlers that mirror backend events into `app.states`, `setup(app)`/`teardown(app)` that wires things. Legacy classes (`NetworkManager`, `PeopleDirectory`, `CloudManager`) are deleted at the end.

**Tech Stack:** Python 3.12+, pytest, uv, existing `pyserial` (modem), `paho-mqtt` (cloud), `requests` (cloud HTTPS). No new runtime dependencies.

**Spec reference:** `docs/superpowers/specs/2026-04-21-phase-a-spine-rewrite-design.md` §3.2 (layout), §5 (entities), §7 (commands), §9.2 (fate), §11.2 (step 4).

**Prerequisite:** Plan 2 executed — `integrations/power/` working, `core/app_shell.py` has `run()`.

---

## File Structure

### Files to create

**Network:**
- `src/yoyopod/backends/network/__init__.py`
- `src/yoyopod/backends/network/modem.py` (moved from `src/yoyopod/network/backend.py`)
- `src/yoyopod/backends/network/ppp.py` (moved from `src/yoyopod/network/ppp.py`)
- `src/yoyopod/backends/network/at_commands.py` (moved)
- `src/yoyopod/backends/network/transport.py` (moved)
- `src/yoyopod/integrations/network/__init__.py` — setup/teardown
- `src/yoyopod/integrations/network/commands.py`
- `src/yoyopod/integrations/network/handlers.py`
- `src/yoyopod/integrations/network/poller.py`
- `tests/integrations/test_network_commands.py`
- `tests/integrations/test_network_handlers.py`
- `tests/integrations/test_network_integration.py`

**Location:**
- `src/yoyopod/backends/location/__init__.py`
- `src/yoyopod/backends/location/gps.py` (moved from `src/yoyopod/network/gps.py`)
- `src/yoyopod/integrations/location/__init__.py`
- `src/yoyopod/integrations/location/commands.py`
- `src/yoyopod/integrations/location/handlers.py`
- `tests/integrations/test_location_commands.py`
- `tests/integrations/test_location_integration.py`

**Contacts:**
- `src/yoyopod/integrations/contacts/__init__.py`
- `src/yoyopod/integrations/contacts/commands.py`
- `src/yoyopod/integrations/contacts/handlers.py`
- `src/yoyopod/integrations/contacts/directory.py` (moved from `src/yoyopod/people/directory.py`)
- `src/yoyopod/integrations/contacts/models.py` (moved from `src/yoyopod/people/models.py`)
- `src/yoyopod/integrations/contacts/cloud_sync.py` (moved from `src/yoyopod/people/cloud_sync.py`)
- `tests/integrations/test_contacts_integration.py`

Contacts is primarily a data service (no external hardware backend), so it lives entirely under `integrations/contacts/` with no corresponding `backends/contacts/` directory.

**Cloud:**
- `src/yoyopod/backends/cloud/__init__.py`
- `src/yoyopod/backends/cloud/mqtt.py` (moved from `src/yoyopod/cloud/mqtt_client.py`)
- `src/yoyopod/backends/cloud/http.py` (moved from `src/yoyopod/cloud/client.py`)
- `src/yoyopod/integrations/cloud/__init__.py`
- `src/yoyopod/integrations/cloud/commands.py`
- `src/yoyopod/integrations/cloud/handlers.py`
- `src/yoyopod/integrations/cloud/models.py` (moved from `src/yoyopod/cloud/models.py`)
- `tests/integrations/test_cloud_integration.py`

### Files to delete (after all four migrations land)

- `src/yoyopod/network/__init__.py`, `manager.py`, `models.py` (moved), `gps.py` (moved), `backend.py` (moved), `ppp.py` (moved), `transport.py` (moved), `at_commands.py` (moved) — the whole `src/yoyopod/network/` package dies.
- `src/yoyopod/people/__init__.py` — the whole package dies.
- `src/yoyopod/cloud/__init__.py`, `manager.py` (replaced) — the whole package dies.

---

## Task 1: Branch state verification

**Files:** none

- [ ] **Step 1.1: Confirm state after Plan 2**

Run:
```bash
git branch --show-current
git log --oneline -10
ls src/yoyopod/integrations/power/
uv run pytest tests/core/ tests/integrations/test_power*.py -q
```

Expected: on `arch/phase-a-spine-rewrite`; Plan 2 commits visible; `integrations/power/` populated; all power + core tests green.

---

## Task 2: Network integration

The network integration owns cellular modem registration, PPP data session, signal strength, and carrier info. GPS is split into its own integration (Task 3) — do NOT include GPS logic here.

**Entities managed:** `network.cellular_registered` (bool), `network.signal_bars` (int 0–5 or None), `network.ppp_up` (bool), `network.carrier` (str), `network.backend_available` (bool).

**Commands:** `enable_ppp`, `disable_ppp`, `refresh_signal`, `set_apn`.

### Subtask 2.1: Scaffold and relocate backend

- [ ] **Step 2.1.1: Create directories and move backend files (excluding gps)**

Run:
```bash
mkdir -p src/yoyopod/backends/network
git mv src/yoyopod/network/backend.py src/yoyopod/backends/network/modem.py
git mv src/yoyopod/network/ppp.py src/yoyopod/backends/network/ppp.py
git mv src/yoyopod/network/at_commands.py src/yoyopod/backends/network/at_commands.py
git mv src/yoyopod/network/transport.py src/yoyopod/backends/network/transport.py
```

Create `src/yoyopod/backends/network/__init__.py`:

```python
"""Cellular modem + PPP adapter (GPS is a separate integration)."""

from __future__ import annotations

from yoyopod.backends.network.modem import ModemBackend
from yoyopod.backends.network.ppp import PPPBackend

__all__ = ["ModemBackend", "PPPBackend"]
```

Update imports in the moved files (`from yoyopod.network.X` → `from yoyopod.backends.network.X`). Grep for stragglers: `grep -rn "from yoyopod.network" src/yoyopod/backends/network/`.

### Subtask 2.2: Write network commands

- [ ] **Step 2.2.1: Create `tests/integrations/test_network_commands.py`**

```python
from yoyopod.integrations.network.commands import (
    DisablePppCommand,
    EnablePppCommand,
    RefreshSignalCommand,
    SetApnCommand,
)


def test_enable_ppp_command() -> None:
    cmd = EnablePppCommand()
    assert cmd is not None


def test_disable_ppp_command() -> None:
    cmd = DisablePppCommand()
    assert cmd is not None


def test_refresh_signal_command() -> None:
    cmd = RefreshSignalCommand()
    assert cmd is not None


def test_set_apn_command() -> None:
    cmd = SetApnCommand(apn="internet.provider.com", username="u", password="p")
    assert cmd.apn == "internet.provider.com"
    assert cmd.username == "u"
    assert cmd.password == "p"


def test_set_apn_command_defaults_empty() -> None:
    cmd = SetApnCommand(apn="internet")
    assert cmd.username == ""
    assert cmd.password == ""
```

- [ ] **Step 2.2.2: Implement `src/yoyopod/integrations/network/commands.py`**

```python
"""Typed commands for the network integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class EnablePppCommand:
    """Request PPP data-session bring-up."""


@dataclass(frozen=True, slots=True)
class DisablePppCommand:
    """Request PPP data-session tear-down."""


@dataclass(frozen=True, slots=True)
class RefreshSignalCommand:
    """One-shot AT query for current signal strength."""


@dataclass(frozen=True, slots=True)
class SetApnCommand:
    """Configure APN credentials for the cellular connection."""

    apn: str
    username: str = ""
    password: str = ""
```

- [ ] **Step 2.2.3: Run tests, format/lint/type, commit**

```bash
uv run pytest tests/integrations/test_network_commands.py -v
uv run black src/yoyopod/integrations/network/commands.py tests/integrations/test_network_commands.py
uv run ruff check src/yoyopod/integrations/network/commands.py tests/integrations/test_network_commands.py
uv run mypy src/yoyopod/integrations/network/commands.py
git add -A
git commit -m "feat(integrations/network): typed commands + backend relocated under backends/network/

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Subtask 2.3: Write network handlers

Handlers translate backend status callbacks into state entity writes.

- [ ] **Step 2.3.1: Create `tests/integrations/test_network_handlers.py`**

```python
from dataclasses import dataclass

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.network.handlers import (
    apply_modem_status_to_state,
    apply_ppp_status_to_state,
    apply_signal_to_state,
)


@dataclass(frozen=True, slots=True)
class _ModemStatus:
    registered: bool
    carrier: str
    network_type: str
    available: bool = True


def test_apply_modem_status_sets_state() -> None:
    app = build_test_app()
    apply_modem_status_to_state(
        app,
        _ModemStatus(registered=True, carrier="T-Mobile", network_type="4G"),
    )
    assert app.states.get_value("network.cellular_registered") is True
    assert app.states.get_value("network.carrier") == "T-Mobile"
    assert app.states.get_value("network.network_type") == "4G"
    assert app.states.get_value("network.backend_available") is True


def test_apply_modem_status_unregistered() -> None:
    app = build_test_app()
    apply_modem_status_to_state(
        app,
        _ModemStatus(registered=False, carrier="", network_type=""),
    )
    assert app.states.get_value("network.cellular_registered") is False


def test_apply_ppp_status() -> None:
    app = build_test_app()
    apply_ppp_status_to_state(app, up=True, reason="session_established")
    assert app.states.get_value("network.ppp_up") is True

    apply_ppp_status_to_state(app, up=False, reason="link_down")
    assert app.states.get_value("network.ppp_up") is False


def test_apply_signal_maps_csq_to_bars() -> None:
    app = build_test_app()
    apply_signal_to_state(app, csq=25, bars=4)
    assert app.states.get_value("network.signal_bars") == 4
    assert app.states.get_value("network.signal_csq") == 25


def test_apply_signal_none_when_no_service() -> None:
    app = build_test_app()
    apply_signal_to_state(app, csq=None, bars=None)
    assert app.states.get_value("network.signal_bars") is None
```

- [ ] **Step 2.3.2: Implement `src/yoyopod/integrations/network/handlers.py`**

```python
"""State-update handlers for the network integration."""

from __future__ import annotations

from typing import Any


def apply_modem_status_to_state(app: Any, status: Any) -> None:
    """Mirror a modem-status snapshot into the state store.

    Expected fields: registered (bool), carrier (str), network_type (str),
    available (bool, defaults True).
    """
    app.states.set("network.cellular_registered", bool(status.registered))
    app.states.set("network.carrier", str(status.carrier))
    app.states.set("network.network_type", str(status.network_type))
    app.states.set("network.backend_available", bool(getattr(status, "available", True)))


def apply_ppp_status_to_state(app: Any, up: bool, reason: str = "") -> None:
    """Set network.ppp_up to the given boolean."""
    app.states.set("network.ppp_up", bool(up), attrs={"reason": reason} if reason else None)


def apply_signal_to_state(app: Any, csq: int | None, bars: int | None) -> None:
    """Set network.signal_bars and network.signal_csq."""
    app.states.set("network.signal_bars", bars)
    app.states.set("network.signal_csq", csq)
```

- [ ] **Step 2.3.3: Run tests, format/lint/type, commit**

```bash
uv run pytest tests/integrations/test_network_handlers.py -v
uv run black src/yoyopod/integrations/network/handlers.py tests/integrations/test_network_handlers.py
uv run ruff check src/yoyopod/integrations/network/handlers.py tests/integrations/test_network_handlers.py
uv run mypy src/yoyopod/integrations/network/handlers.py
git add -A
git commit -m "feat(integrations/network): state-update handlers

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Subtask 2.4: Write network poller and integration setup

- [ ] **Step 2.4.1: Create `src/yoyopod/integrations/network/poller.py`**

The network poller queries the modem for registration, signal, and carrier at a fixed cadence. Follow the same structure as the power poller (`src/yoyopod/integrations/power/poller.py`). Thread-based, stop event, `run_on_main` marshalling.

```python
"""Background poller for modem registration + signal strength."""

from __future__ import annotations

import threading
from typing import Any

from loguru import logger

from yoyopod.integrations.network.handlers import (
    apply_modem_status_to_state,
    apply_signal_to_state,
)


class NetworkPoller:
    def __init__(self, app: Any, backend: Any, interval_seconds: float = 15.0) -> None:
        self._app = app
        self._backend = backend
        self._interval = max(0.01, float(interval_seconds))
        self._thread: threading.Thread | None = None
        self._stop = threading.Event()

    def start(self) -> None:
        if self._thread is not None and self._thread.is_alive():
            return
        self._stop.clear()
        self._thread = threading.Thread(target=self._run, daemon=True, name="network-poller")
        self._thread.start()

    def stop(self) -> None:
        self._stop.set()
        if self._thread is not None:
            self._thread.join(timeout=1.0)
        self._thread = None

    def _run(self) -> None:
        while not self._stop.is_set():
            try:
                status = self._backend.get_status()
                signal = self._backend.get_signal()
            except Exception as exc:
                logger.error("NetworkPoller query failed: {}", exc)
                self._stop.wait(self._interval)
                continue

            self._app.scheduler.run_on_main(
                lambda s=status, sig=signal: self._apply(s, sig)
            )
            self._stop.wait(self._interval)

    def _apply(self, status: Any, signal: Any) -> None:
        apply_modem_status_to_state(self._app, status)
        apply_signal_to_state(
            self._app,
            csq=getattr(signal, "csq", None),
            bars=getattr(signal, "bars", None),
        )
```

- [ ] **Step 2.4.2: Create `src/yoyopod/integrations/network/__init__.py`**

```python
"""Network integration: cellular registration, PPP, signal, carrier.

GPS is a separate integration (yoyopod.integrations.location).
"""

from __future__ import annotations

from typing import Any

from loguru import logger

from yoyopod.integrations.network.commands import (
    DisablePppCommand,
    EnablePppCommand,
    RefreshSignalCommand,
    SetApnCommand,
)
from yoyopod.integrations.network.handlers import (
    apply_ppp_status_to_state,
    apply_signal_to_state,
)
from yoyopod.integrations.network.poller import NetworkPoller

_STATE_KEY = "_network_integration"


def setup(app: Any, backend: Any | None = None) -> None:
    """Wire the network integration."""
    if backend is None:
        from yoyopod.backends.network import ModemBackend, PPPBackend
        modem = ModemBackend(app.config.network)
        ppp = PPPBackend(app.config.network, modem=modem)
        backend = _ModemPPPAdapter(modem=modem, ppp=ppp)

    poller = NetworkPoller(
        app=app,
        backend=backend,
        interval_seconds=float(app.config.network.poll_interval_seconds),
    )

    def handle_enable_ppp(_cmd: EnablePppCommand) -> None:
        logger.info("Network.enable_ppp")
        try:
            backend.enable_ppp()
            apply_ppp_status_to_state(app, up=True, reason="enabled")
        except Exception as exc:
            logger.error("enable_ppp failed: {}", exc)
            apply_ppp_status_to_state(app, up=False, reason=str(exc))

    def handle_disable_ppp(_cmd: DisablePppCommand) -> None:
        logger.info("Network.disable_ppp")
        try:
            backend.disable_ppp()
            apply_ppp_status_to_state(app, up=False, reason="disabled")
        except Exception as exc:
            logger.error("disable_ppp failed: {}", exc)

    def handle_refresh_signal(_cmd: RefreshSignalCommand) -> None:
        signal = backend.get_signal()
        apply_signal_to_state(
            app,
            csq=getattr(signal, "csq", None),
            bars=getattr(signal, "bars", None),
        )

    def handle_set_apn(cmd: SetApnCommand) -> None:
        backend.set_apn(apn=cmd.apn, username=cmd.username, password=cmd.password)

    app.services.register("network", "enable_ppp", handle_enable_ppp)
    app.services.register("network", "disable_ppp", handle_disable_ppp)
    app.services.register("network", "refresh_signal", handle_refresh_signal)
    app.services.register("network", "set_apn", handle_set_apn)

    poller.start()
    setattr(app, _STATE_KEY, {"backend": backend, "poller": poller})


def teardown(app: Any) -> None:
    state = getattr(app, _STATE_KEY, None)
    if state is None:
        return
    try:
        state["poller"].stop()
    except Exception as exc:
        logger.error("NetworkPoller.stop: {}", exc)
    try:
        close = getattr(state["backend"], "close", None)
        if callable(close):
            close()
    except Exception as exc:
        logger.error("Network backend close: {}", exc)
    delattr(app, _STATE_KEY)


class _ModemPPPAdapter:
    """Adapter that unifies ModemBackend + PPPBackend behind one object."""

    def __init__(self, modem: Any, ppp: Any) -> None:
        self._modem = modem
        self._ppp = ppp

    def get_status(self) -> Any:
        return self._modem.get_status()

    def get_signal(self) -> Any:
        return self._modem.get_signal()

    def enable_ppp(self) -> None:
        self._ppp.bring_up()

    def disable_ppp(self) -> None:
        self._ppp.tear_down()

    def set_apn(self, apn: str, username: str, password: str) -> None:
        self._modem.set_apn(apn=apn, username=username, password=password)

    def close(self) -> None:
        try:
            self._ppp.tear_down()
        except Exception:
            pass
        close = getattr(self._modem, "close", None)
        if callable(close):
            close()
```

- [ ] **Step 2.4.3: Create integration test `tests/integrations/test_network_integration.py`**

```python
import time
from dataclasses import dataclass

import pytest

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.network import setup as setup_network, teardown as teardown_network
from yoyopod.integrations.network.commands import (
    DisablePppCommand,
    EnablePppCommand,
    RefreshSignalCommand,
    SetApnCommand,
)


@dataclass
class _FakeNetworkBackend:
    registered: bool = True
    carrier: str = "TestCarrier"
    network_type: str = "4G"
    csq: int = 20
    bars: int = 3
    ppp_commands: list[str] = None
    apn_calls: list[tuple[str, str, str]] = None

    def __post_init__(self):
        self.ppp_commands = []
        self.apn_calls = []

    def get_status(self):
        @dataclass(frozen=True, slots=True)
        class _S:
            registered: bool
            carrier: str
            network_type: str
            available: bool = True

        return _S(self.registered, self.carrier, self.network_type)

    def get_signal(self):
        @dataclass(frozen=True, slots=True)
        class _Sig:
            csq: int
            bars: int

        return _Sig(self.csq, self.bars)

    def enable_ppp(self):
        self.ppp_commands.append("up")

    def disable_ppp(self):
        self.ppp_commands.append("down")

    def set_apn(self, apn: str, username: str = "", password: str = ""):
        self.apn_calls.append((apn, username, password))

    def close(self):
        pass


@pytest.fixture
def app_with_network():
    app = build_test_app()
    backend = _FakeNetworkBackend()
    app.config = type("C", (), {"network": type("NC", (), {"poll_interval_seconds": 0.05})()})()
    app.register_integration(
        "network",
        setup=lambda a: setup_network(a, backend=backend),
        teardown=lambda a: teardown_network(a),
    )
    app.setup()
    yield app, backend
    app.stop()


def test_setup_registers_commands(app_with_network):
    app, _ = app_with_network
    pairs = set(app.services.registered())
    assert ("network", "enable_ppp") in pairs
    assert ("network", "disable_ppp") in pairs
    assert ("network", "refresh_signal") in pairs
    assert ("network", "set_apn") in pairs


def test_poller_mirrors_modem_status(app_with_network):
    app, _ = app_with_network
    time.sleep(0.2)
    app.drain()
    assert app.states.get_value("network.cellular_registered") is True
    assert app.states.get_value("network.carrier") == "TestCarrier"


def test_enable_ppp_command_updates_state(app_with_network):
    app, backend = app_with_network
    app.services.call("network", "enable_ppp", EnablePppCommand())
    assert backend.ppp_commands == ["up"]
    assert app.states.get_value("network.ppp_up") is True


def test_disable_ppp_command(app_with_network):
    app, backend = app_with_network
    app.services.call("network", "disable_ppp", DisablePppCommand())
    assert backend.ppp_commands == ["down"]
    assert app.states.get_value("network.ppp_up") is False


def test_set_apn_command(app_with_network):
    app, backend = app_with_network
    app.services.call("network", "set_apn", SetApnCommand(apn="a", username="u", password="p"))
    assert backend.apn_calls == [("a", "u", "p")]


def test_refresh_signal_command(app_with_network):
    app, _ = app_with_network
    app.services.call("network", "refresh_signal", RefreshSignalCommand())
    assert app.states.get_value("network.signal_bars") == 3
    assert app.states.get_value("network.signal_csq") == 20
```

- [ ] **Step 2.4.4: Run everything, commit**

```bash
uv run pytest tests/integrations/test_network_integration.py -v
uv run black src/yoyopod/integrations/network/ tests/integrations/test_network_integration.py
uv run ruff check src/yoyopod/integrations/network/ tests/integrations/test_network_integration.py
uv run mypy src/yoyopod/integrations/network/
git add -A
git commit -m "feat(integrations/network): poller, setup/teardown, end-to-end test

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Location integration (GPS split out)

GPS is the sole responsibility of the location integration. Strip it out of the legacy network package and treat it as its own domain.

**Entities:** `location.fix` (LocationFix | None; attrs: lat, lng, altitude, speed_mps, last_fix_at), `location.backend_available` (bool).

**Commands:** `request_fix`, `enable_gps`, `disable_gps`.

### Subtask 3.1: Move GPS backend

- [ ] **Step 3.1.1: Relocate**

```bash
mkdir -p src/yoyopod/backends/location
git mv src/yoyopod/network/gps.py src/yoyopod/backends/location/gps.py
```

Update imports inside `src/yoyopod/backends/location/gps.py` (`from yoyopod.network.*` → `from yoyopod.backends.location.*` / `from yoyopod.backends.network.*` as appropriate — GPS typically shares the modem transport, so it may still need `from yoyopod.backends.network.modem import ModemBackend` or similar).

Create `src/yoyopod/backends/location/__init__.py`:

```python
"""GPS backend (shares the cellular modem serial transport)."""

from __future__ import annotations

from yoyopod.backends.location.gps import GpsBackend

__all__ = ["GpsBackend"]
```

### Subtask 3.2: Commands, handlers, integration

- [ ] **Step 3.2.1: Create `tests/integrations/test_location_commands.py`**

```python
from yoyopod.integrations.location.commands import (
    DisableGpsCommand,
    EnableGpsCommand,
    RequestFixCommand,
)


def test_request_fix_argless():
    assert RequestFixCommand() is not None


def test_enable_gps_argless():
    assert EnableGpsCommand() is not None


def test_disable_gps_argless():
    assert DisableGpsCommand() is not None
```

- [ ] **Step 3.2.2: Implement `src/yoyopod/integrations/location/commands.py`**

```python
"""Typed commands for the location integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class RequestFixCommand:
    """Ask the backend for the latest GPS fix; applies result to state."""


@dataclass(frozen=True, slots=True)
class EnableGpsCommand:
    """Bring up the GPS receiver."""


@dataclass(frozen=True, slots=True)
class DisableGpsCommand:
    """Power down the GPS receiver to save battery."""
```

- [ ] **Step 3.2.3: Implement `src/yoyopod/integrations/location/handlers.py` and `__init__.py`**

Create `src/yoyopod/integrations/location/handlers.py`:

```python
"""State-update handler for location.fix."""

from __future__ import annotations

import time
from dataclasses import dataclass
from typing import Any


@dataclass(frozen=True, slots=True)
class LocationFix:
    lat: float
    lng: float
    altitude: float
    speed_mps: float
    last_fix_at: float


def apply_fix_to_state(app: Any, fix: Any | None, reason: str = "") -> None:
    """Mirror a GPS fix into state. None means 'no current fix'."""
    if fix is None:
        app.states.set("location.fix", None, attrs={"no_fix_reason": reason})
        return

    lf = LocationFix(
        lat=float(fix.lat),
        lng=float(fix.lng),
        altitude=float(getattr(fix, "altitude", 0.0)),
        speed_mps=float(getattr(fix, "speed", getattr(fix, "speed_mps", 0.0))),
        last_fix_at=time.time(),
    )
    app.states.set("location.fix", lf)


def apply_availability_to_state(app: Any, available: bool, reason: str = "") -> None:
    app.states.set(
        "location.backend_available",
        bool(available),
        attrs={"reason": reason} if reason else None,
    )
```

Create `src/yoyopod/integrations/location/__init__.py`:

```python
"""Location (GPS) integration."""

from __future__ import annotations

from typing import Any

from loguru import logger

from yoyopod.integrations.location.commands import (
    DisableGpsCommand,
    EnableGpsCommand,
    RequestFixCommand,
)
from yoyopod.integrations.location.handlers import (
    apply_availability_to_state,
    apply_fix_to_state,
)

_STATE_KEY = "_location_integration"


def setup(app: Any, backend: Any | None = None) -> None:
    if backend is None:
        from yoyopod.backends.location import GpsBackend
        backend = GpsBackend(app.config.network)  # shares network config (modem serial)

    def handle_request_fix(_cmd: RequestFixCommand) -> None:
        try:
            fix = backend.get_fix()
        except Exception as exc:
            logger.error("GPS.get_fix failed: {}", exc)
            apply_fix_to_state(app, None, reason=str(exc))
            return
        apply_fix_to_state(app, fix, reason="" if fix else "no_fix")

    def handle_enable_gps(_cmd: EnableGpsCommand) -> None:
        try:
            backend.enable()
            apply_availability_to_state(app, True)
        except Exception as exc:
            logger.error("GPS.enable failed: {}", exc)
            apply_availability_to_state(app, False, reason=str(exc))

    def handle_disable_gps(_cmd: DisableGpsCommand) -> None:
        try:
            backend.disable()
            apply_availability_to_state(app, False, reason="disabled")
        except Exception as exc:
            logger.error("GPS.disable failed: {}", exc)

    app.services.register("location", "request_fix", handle_request_fix)
    app.services.register("location", "enable_gps", handle_enable_gps)
    app.services.register("location", "disable_gps", handle_disable_gps)

    setattr(app, _STATE_KEY, {"backend": backend})


def teardown(app: Any) -> None:
    state = getattr(app, _STATE_KEY, None)
    if state is None:
        return
    close = getattr(state["backend"], "close", None)
    if callable(close):
        try:
            close()
        except Exception as exc:
            logger.error("GpsBackend.close: {}", exc)
    delattr(app, _STATE_KEY)
```

- [ ] **Step 3.2.4: Create `tests/integrations/test_location_integration.py`**

```python
from dataclasses import dataclass

import pytest

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.location import setup as setup_location, teardown as teardown_location
from yoyopod.integrations.location.commands import (
    DisableGpsCommand,
    EnableGpsCommand,
    RequestFixCommand,
)


@dataclass
class _FakeGpsBackend:
    fix_lat: float = 48.1
    fix_lng: float = 11.5
    return_none: bool = False
    enabled: bool = False

    def get_fix(self):
        if self.return_none:
            return None

        @dataclass(frozen=True, slots=True)
        class _F:
            lat: float
            lng: float
            altitude: float = 0.0
            speed: float = 0.0

        return _F(self.fix_lat, self.fix_lng)

    def enable(self):
        self.enabled = True

    def disable(self):
        self.enabled = False

    def close(self):
        pass


@pytest.fixture
def app_with_location():
    app = build_test_app()
    backend = _FakeGpsBackend()
    app.config = type("C", (), {"network": object()})()
    app.register_integration(
        "location",
        setup=lambda a: setup_location(a, backend=backend),
        teardown=lambda a: teardown_location(a),
    )
    app.setup()
    yield app, backend
    app.stop()


def test_setup_registers_commands(app_with_location):
    app, _ = app_with_location
    pairs = set(app.services.registered())
    assert ("location", "request_fix") in pairs
    assert ("location", "enable_gps") in pairs
    assert ("location", "disable_gps") in pairs


def test_request_fix_applies_to_state(app_with_location):
    app, _ = app_with_location
    app.services.call("location", "request_fix", RequestFixCommand())

    fix = app.states.get_value("location.fix")
    assert fix is not None
    assert fix.lat == 48.1
    assert fix.lng == 11.5


def test_request_fix_none_sets_attr_reason(app_with_location):
    app, backend = app_with_location
    backend.return_none = True
    app.services.call("location", "request_fix", RequestFixCommand())
    sv = app.states.get("location.fix")
    assert sv.value is None
    assert sv.attrs.get("no_fix_reason") == "no_fix"


def test_enable_disable(app_with_location):
    app, backend = app_with_location
    app.services.call("location", "enable_gps", EnableGpsCommand())
    assert backend.enabled is True
    assert app.states.get_value("location.backend_available") is True

    app.services.call("location", "disable_gps", DisableGpsCommand())
    assert backend.enabled is False
    assert app.states.get_value("location.backend_available") is False
```

- [ ] **Step 3.2.5: Run tests, format/lint/type, commit**

```bash
uv run pytest tests/integrations/test_location_commands.py tests/integrations/test_location_integration.py -v
uv run black src/yoyopod/integrations/location/ src/yoyopod/backends/location/ tests/integrations/test_location_commands.py tests/integrations/test_location_integration.py
uv run ruff check src/yoyopod/integrations/location/ src/yoyopod/backends/location/ tests/integrations/test_location_commands.py tests/integrations/test_location_integration.py
uv run mypy src/yoyopod/integrations/location/ src/yoyopod/backends/location/
git add -A
git commit -m "feat(integrations/location): split GPS out of network into its own integration

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Contacts integration (data-only, no backend)

The contacts integration owns the people directory. It has no external backend — all data is local (files + optional cloud sync logic that already lives in `people/cloud_sync.py`).

**Entities:** `contacts.unread_voice_notes` (int; attrs: by_address: dict), `contacts.people_count` (int).

**Commands:** `lookup_by_address` (returns contact or None), `reload`, `mark_voice_notes_seen`.

### Subtask 4.1: Relocate the people directory

- [ ] **Step 4.1.1: Move files**

```bash
mkdir -p src/yoyopod/integrations/contacts
git mv src/yoyopod/people/directory.py src/yoyopod/integrations/contacts/directory.py
git mv src/yoyopod/people/models.py src/yoyopod/integrations/contacts/models.py
git mv src/yoyopod/people/cloud_sync.py src/yoyopod/integrations/contacts/cloud_sync.py
```

Update imports inside the moved files (`from yoyopod.people.models` → `from yoyopod.integrations.contacts.models`). Grep for remaining references: `grep -rn "from yoyopod.people" src/yoyopod/integrations/contacts/`.

### Subtask 4.2: Commands + setup

- [ ] **Step 4.2.1: Create `src/yoyopod/integrations/contacts/commands.py`**

```python
"""Typed commands for the contacts integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class LookupByAddressCommand:
    """Look up a contact by SIP address; returns Contact | None."""

    address: str


@dataclass(frozen=True, slots=True)
class ReloadCommand:
    """Re-read the contacts file from disk."""


@dataclass(frozen=True, slots=True)
class MarkVoiceNotesSeenCommand:
    """Clear unread voice-note marker for the given SIP address."""

    address: str
```

- [ ] **Step 4.2.2: Create `src/yoyopod/integrations/contacts/__init__.py`**

```python
"""Contacts integration: people directory + voice-note seen tracking."""

from __future__ import annotations

from typing import Any

from loguru import logger

from yoyopod.integrations.contacts.commands import (
    LookupByAddressCommand,
    MarkVoiceNotesSeenCommand,
    ReloadCommand,
)
from yoyopod.integrations.contacts.directory import PeopleDirectory

_STATE_KEY = "_contacts_integration"


def setup(app: Any, directory: PeopleDirectory | None = None) -> None:
    if directory is None:
        directory = PeopleDirectory(app.config.people)
        directory.load()

    def _refresh_counts() -> None:
        app.states.set("contacts.people_count", directory.count())
        app.states.set(
            "contacts.unread_voice_notes",
            directory.unread_voice_note_count(),
            attrs={"by_address": dict(directory.unread_voice_notes_by_address())},
        )

    def handle_lookup(cmd: LookupByAddressCommand):
        return directory.get_contact_by_address(cmd.address)

    def handle_reload(_cmd: ReloadCommand) -> None:
        logger.info("Contacts.reload")
        directory.load()
        _refresh_counts()

    def handle_mark_seen(cmd: MarkVoiceNotesSeenCommand) -> None:
        directory.mark_voice_notes_seen(cmd.address)
        _refresh_counts()

    app.services.register("contacts", "lookup_by_address", handle_lookup)
    app.services.register("contacts", "reload", handle_reload)
    app.services.register("contacts", "mark_voice_notes_seen", handle_mark_seen)

    _refresh_counts()
    setattr(app, _STATE_KEY, {"directory": directory})


def teardown(app: Any) -> None:
    state = getattr(app, _STATE_KEY, None)
    if state is None:
        return
    delattr(app, _STATE_KEY)
```

- [ ] **Step 4.2.3: Create `tests/integrations/test_contacts_integration.py`**

```python
from dataclasses import dataclass

import pytest

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.contacts import setup as setup_contacts, teardown as teardown_contacts
from yoyopod.integrations.contacts.commands import (
    LookupByAddressCommand,
    MarkVoiceNotesSeenCommand,
    ReloadCommand,
)


@dataclass
class _FakeContact:
    display_name: str
    sip_address: str


class _FakeDirectory:
    def __init__(self):
        self._people = {"sip:alice@x": _FakeContact("Alice", "sip:alice@x")}
        self._unread = {"sip:alice@x": 2}
        self.load_calls = 0

    def load(self):
        self.load_calls += 1

    def count(self):
        return len(self._people)

    def get_contact_by_address(self, address):
        return self._people.get(address)

    def unread_voice_note_count(self):
        return sum(self._unread.values())

    def unread_voice_notes_by_address(self):
        return dict(self._unread)

    def mark_voice_notes_seen(self, address):
        self._unread.pop(address, None)


@pytest.fixture
def app_with_contacts():
    app = build_test_app()
    directory = _FakeDirectory()
    app.register_integration(
        "contacts",
        setup=lambda a: setup_contacts(a, directory=directory),
        teardown=lambda a: teardown_contacts(a),
    )
    app.setup()
    yield app, directory
    app.stop()


def test_setup_seeds_state(app_with_contacts):
    app, _ = app_with_contacts
    assert app.states.get_value("contacts.people_count") == 1
    assert app.states.get_value("contacts.unread_voice_notes") == 2


def test_lookup_returns_contact(app_with_contacts):
    app, _ = app_with_contacts
    contact = app.services.call(
        "contacts", "lookup_by_address", LookupByAddressCommand(address="sip:alice@x")
    )
    assert contact is not None
    assert contact.display_name == "Alice"


def test_lookup_missing_returns_none(app_with_contacts):
    app, _ = app_with_contacts
    contact = app.services.call(
        "contacts", "lookup_by_address", LookupByAddressCommand(address="sip:unknown")
    )
    assert contact is None


def test_reload(app_with_contacts):
    app, directory = app_with_contacts
    app.services.call("contacts", "reload", ReloadCommand())
    assert directory.load_calls == 1


def test_mark_voice_notes_seen_updates_state(app_with_contacts):
    app, _ = app_with_contacts
    app.services.call(
        "contacts",
        "mark_voice_notes_seen",
        MarkVoiceNotesSeenCommand(address="sip:alice@x"),
    )
    assert app.states.get_value("contacts.unread_voice_notes") == 0
```

- [ ] **Step 4.2.4: Run tests, format/lint/type, commit**

```bash
uv run pytest tests/integrations/test_contacts_integration.py -v
uv run black src/yoyopod/integrations/contacts/ tests/integrations/test_contacts_integration.py
uv run ruff check src/yoyopod/integrations/contacts/ tests/integrations/test_contacts_integration.py
uv run mypy src/yoyopod/integrations/contacts/
git add -A
git commit -m "feat(integrations/contacts): relocate people directory, add service layer

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Cloud integration

The cloud integration subscribes to `StateChangedEvent` and publishes selected state changes to MQTT — the first concrete demonstration of A+3's "add new observers for free" property. Also owns HTTPS auth/token refresh and config polling.

**Entities:** `cloud.mqtt_connected` (bool; attrs: reason, last_sync_at).

**Commands:** `sync_now`, `publish_telemetry`.

### Subtask 5.1: Relocate cloud backend

- [ ] **Step 5.1.1: Move files**

```bash
mkdir -p src/yoyopod/backends/cloud
git mv src/yoyopod/cloud/mqtt_client.py src/yoyopod/backends/cloud/mqtt.py
git mv src/yoyopod/cloud/client.py src/yoyopod/backends/cloud/http.py
git mv src/yoyopod/cloud/models.py src/yoyopod/integrations/cloud/models.py
mkdir -p src/yoyopod/integrations/cloud
```

Update imports. Create `src/yoyopod/backends/cloud/__init__.py`:

```python
"""Cloud backends: MQTT (telemetry + remote commands) + HTTPS (auth, config, provisioning)."""

from __future__ import annotations

from yoyopod.backends.cloud.http import CloudHttpClient
from yoyopod.backends.cloud.mqtt import DeviceMqttClient

__all__ = ["CloudHttpClient", "DeviceMqttClient"]
```

### Subtask 5.2: Commands + setup

- [ ] **Step 5.2.1: Create `src/yoyopod/integrations/cloud/commands.py`**

```python
"""Typed commands for the cloud integration."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class SyncNowCommand:
    """Trigger an HTTPS config-sync roundtrip on demand."""


@dataclass(frozen=True, slots=True)
class PublishTelemetryCommand:
    """Publish an explicit telemetry payload to MQTT."""

    topic_suffix: str
    payload: dict
```

- [ ] **Step 5.2.2: Create `src/yoyopod/integrations/cloud/handlers.py`**

```python
"""Cloud integration handlers: MQTT status + state-change forwarding."""

from __future__ import annotations

import json
import time
from typing import Any

from loguru import logger

from yoyopod.core.events import StateChangedEvent


_TELEMETRY_ENTITIES = {
    "power.battery_percent",
    "power.charging",
    "network.cellular_registered",
    "network.signal_bars",
    "network.ppp_up",
    "location.fix",
    "call.state",
    "music.state",
}


def apply_mqtt_status_to_state(app: Any, connected: bool, reason: str = "") -> None:
    attrs = {"reason": reason, "last_sync_at": time.time()} if reason else {"last_sync_at": time.time()}
    app.states.set("cloud.mqtt_connected", bool(connected), attrs=attrs)


def build_forwarder(app: Any, mqtt_client: Any) -> Any:
    """Build a StateChangedEvent subscriber that forwards interesting entities to MQTT."""

    def on_state_changed(ev: StateChangedEvent) -> None:
        if ev.entity not in _TELEMETRY_ENTITIES:
            return
        if not app.states.get_value("cloud.mqtt_connected"):
            return
        try:
            payload = {
                "entity": ev.entity,
                "value": _serialisable(ev.new.value),
                "attrs": _serialisable_dict(ev.new.attrs),
                "ts": ev.new.last_changed_at,
            }
            mqtt_client.publish(f"yoyopod/telemetry/{ev.entity}", json.dumps(payload), qos=0)
        except Exception as exc:
            logger.error("cloud forwarder MQTT publish failed: {}", exc)

    return on_state_changed


def _serialisable(value: Any) -> Any:
    if value is None or isinstance(value, (bool, int, float, str)):
        return value
    if hasattr(value, "__dict__"):
        return {k: _serialisable(v) for k, v in vars(value).items()}
    return str(value)


def _serialisable_dict(d: dict) -> dict:
    return {k: _serialisable(v) for k, v in d.items()}
```

- [ ] **Step 5.2.3: Create `src/yoyopod/integrations/cloud/__init__.py`**

```python
"""Cloud integration: MQTT telemetry, HTTPS config sync, remote commands."""

from __future__ import annotations

from typing import Any

from loguru import logger

from yoyopod.core.events import StateChangedEvent
from yoyopod.integrations.cloud.commands import (
    PublishTelemetryCommand,
    SyncNowCommand,
)
from yoyopod.integrations.cloud.handlers import (
    apply_mqtt_status_to_state,
    build_forwarder,
)

_STATE_KEY = "_cloud_integration"


def setup(
    app: Any,
    mqtt_client: Any | None = None,
    http_client: Any | None = None,
) -> None:
    if mqtt_client is None:
        from yoyopod.backends.cloud import DeviceMqttClient
        mqtt_client = DeviceMqttClient(app.config.cloud)
    if http_client is None:
        from yoyopod.backends.cloud import CloudHttpClient
        http_client = CloudHttpClient(app.config.cloud)

    def on_mqtt_connect() -> None:
        app.scheduler.run_on_main(lambda: apply_mqtt_status_to_state(app, True, "connected"))

    def on_mqtt_disconnect(reason: str = "") -> None:
        app.scheduler.run_on_main(lambda: apply_mqtt_status_to_state(app, False, reason))

    mqtt_client.on_connect(on_mqtt_connect)
    mqtt_client.on_disconnect(on_mqtt_disconnect)

    forwarder = build_forwarder(app, mqtt_client)
    app.bus.subscribe(StateChangedEvent, forwarder)

    mqtt_client.start()

    def handle_sync_now(_cmd: SyncNowCommand) -> None:
        try:
            config = http_client.sync_config()
            app.states.set("cloud.last_sync_at", time.time(), attrs={})
        except Exception as exc:
            logger.error("cloud sync_now failed: {}", exc)

    def handle_publish_telemetry(cmd: PublishTelemetryCommand) -> None:
        try:
            import json
            mqtt_client.publish(
                f"yoyopod/telemetry/{cmd.topic_suffix}",
                json.dumps(cmd.payload),
                qos=0,
            )
        except Exception as exc:
            logger.error("publish_telemetry failed: {}", exc)

    import time  # avoid circular import in handler closures above

    app.services.register("cloud", "sync_now", handle_sync_now)
    app.services.register("cloud", "publish_telemetry", handle_publish_telemetry)

    setattr(app, _STATE_KEY, {"mqtt": mqtt_client, "http": http_client})


def teardown(app: Any) -> None:
    state = getattr(app, _STATE_KEY, None)
    if state is None:
        return
    try:
        state["mqtt"].stop()
    except Exception as exc:
        logger.error("mqtt stop: {}", exc)
    delattr(app, _STATE_KEY)
```

- [ ] **Step 5.2.4: Create `tests/integrations/test_cloud_integration.py`**

```python
import json
from dataclasses import dataclass, field
from typing import Callable

import pytest

from yoyopod.core.testing import build_test_app
from yoyopod.integrations.cloud import setup as setup_cloud, teardown as teardown_cloud
from yoyopod.integrations.cloud.commands import (
    PublishTelemetryCommand,
    SyncNowCommand,
)


@dataclass
class _FakeMqtt:
    _connect_cb: Callable[[], None] | None = None
    _disconnect_cb: Callable[[str], None] | None = None
    started: bool = False
    published: list[tuple[str, str]] = field(default_factory=list)

    def on_connect(self, cb):
        self._connect_cb = cb

    def on_disconnect(self, cb):
        self._disconnect_cb = cb

    def start(self):
        self.started = True
        if self._connect_cb:
            self._connect_cb()

    def stop(self):
        self.started = False

    def publish(self, topic, payload, qos=0):
        self.published.append((topic, payload))


@dataclass
class _FakeHttp:
    synced: int = 0

    def sync_config(self):
        self.synced += 1
        return {}


@pytest.fixture
def app_with_cloud():
    app = build_test_app()
    mqtt = _FakeMqtt()
    http = _FakeHttp()
    app.register_integration(
        "cloud",
        setup=lambda a: setup_cloud(a, mqtt_client=mqtt, http_client=http),
        teardown=lambda a: teardown_cloud(a),
    )
    app.setup()
    app.drain()
    yield app, mqtt, http
    app.stop()


def test_setup_connects_mqtt_and_sets_state(app_with_cloud):
    app, mqtt, _ = app_with_cloud
    assert mqtt.started is True
    assert app.states.get_value("cloud.mqtt_connected") is True


def test_state_changes_forward_to_mqtt(app_with_cloud):
    app, mqtt, _ = app_with_cloud
    app.states.set("power.battery_percent", 77)
    app.drain()

    telemetry = [p for p in mqtt.published if "power.battery_percent" in p[0]]
    assert len(telemetry) == 1
    payload = json.loads(telemetry[0][1])
    assert payload["entity"] == "power.battery_percent"
    assert payload["value"] == 77


def test_non_telemetry_entity_not_forwarded(app_with_cloud):
    app, mqtt, _ = app_with_cloud
    before = len(mqtt.published)
    app.states.set("contacts.people_count", 5)
    app.drain()
    after = len(mqtt.published)
    assert after == before  # contacts.people_count is not in _TELEMETRY_ENTITIES


def test_sync_now_command(app_with_cloud):
    app, _, http = app_with_cloud
    app.services.call("cloud", "sync_now", SyncNowCommand())
    assert http.synced == 1


def test_publish_telemetry_command(app_with_cloud):
    app, mqtt, _ = app_with_cloud
    app.services.call(
        "cloud",
        "publish_telemetry",
        PublishTelemetryCommand(topic_suffix="custom", payload={"x": 1}),
    )
    custom = [p for p in mqtt.published if "custom" in p[0]]
    assert len(custom) == 1
    assert json.loads(custom[0][1]) == {"x": 1}
```

- [ ] **Step 5.2.5: Run tests, format/lint/type, commit**

```bash
uv run pytest tests/integrations/test_cloud_integration.py -v
uv run black src/yoyopod/integrations/cloud/ src/yoyopod/backends/cloud/ tests/integrations/test_cloud_integration.py
uv run ruff check src/yoyopod/integrations/cloud/ src/yoyopod/backends/cloud/ tests/integrations/test_cloud_integration.py
uv run mypy src/yoyopod/integrations/cloud/ src/yoyopod/backends/cloud/
git add -A
git commit -m "feat(integrations/cloud): MQTT telemetry + HTTPS sync via StateChangedEvent subscriber

First demonstration of A+3's 'add observers for free' pattern — the cloud
integration subscribes to StateChangedEvent and forwards selected entities
to MQTT with zero changes to power/network/location integrations.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Retire `NetworkManager`, `PeopleDirectory` (legacy exports), `CloudManager`

- [ ] **Step 6.1: Enumerate remaining consumers**

```bash
grep -rn "NetworkManager\|PeopleDirectory\|CloudManager" src/ tests/
grep -rn "from yoyopod.network import\|from yoyopod.people import\|from yoyopod.cloud import" src/ tests/
```

For each result outside of docs: either the file is still a legacy shell (`src/yoyopod/app.py`, its remaining imports) that will be rewritten in Plan 8 — stub to compile; or the file is a test that's already slated for deletion per the Phase A spec §12.1 — delete it.

- [ ] **Step 6.2: Delete legacy package files**

```bash
git rm src/yoyopod/network/manager.py
git rm src/yoyopod/network/models.py
git rm src/yoyopod/network/__init__.py
git rm src/yoyopod/people/__init__.py
git rm src/yoyopod/cloud/manager.py
git rm src/yoyopod/cloud/__init__.py
```

Any `__init__.py` reference that no longer exists — skip.

- [ ] **Step 6.3: Update `src/yoyopod/app.py` imports**

Remove `from yoyopod.network import NetworkManager`, `from yoyopod.people import PeopleDirectory`, `from yoyopod.cloud import CloudManager`, and the corresponding instance variables/construction in `app.py`. Leave comments pointing at the new integrations. `app.py` is still the legacy shell and will be rewritten in Plan 8.

- [ ] **Step 6.4: Run CI gate**

```bash
uv run python scripts/quality.py ci
```

Expected: all green.

- [ ] **Step 6.5: Commit**

```bash
git add -A
git commit -m "refactor: delete legacy NetworkManager, PeopleDirectory, CloudManager

All four domains migrated to integrations/. Legacy packages dies. Tests
updated; app.py still the legacy shell (rewritten in Plan 8).

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Final verification

- [ ] **Step 7.1: Confirm structure**

```bash
ls src/yoyopod/integrations/
ls src/yoyopod/backends/
```

Expected `integrations/`: `__init__.py`, `power/`, `network/`, `location/`, `contacts/`, `cloud/`.
Expected `backends/`: `__init__.py`, `power/`, `network/`, `location/`, `cloud/`.

- [ ] **Step 7.2: Full CI gate**

```bash
uv run python scripts/quality.py ci
```

Expected: all green.

- [ ] **Step 7.3: Confirm no legacy references in non-doc paths**

```bash
git grep -l "from yoyopod.network\|from yoyopod.people\|from yoyopod.cloud"
```

Expected: matches only in `docs/` (spec/plan references) and archive. Anything else — fix.

- [ ] **Step 7.4: Commit count**

```bash
git log --oneline arch/phase-a-spine-rewrite ^main
```

Expected ~12-14 new commits on top of Plan 2 (one per subtask plus cleanup commit).

---

## Definition of Done

- `integrations/network/`, `integrations/location/`, `integrations/contacts/`, `integrations/cloud/` all populated with setup/teardown, commands, handlers, tests.
- `backends/network/`, `backends/location/`, `backends/cloud/` populated with relocated adapter code.
- Legacy `src/yoyopod/network/`, `src/yoyopod/people/`, `src/yoyopod/cloud/` packages deleted.
- Cloud integration demonstrates the "free observability" property: state changes in `power.*`, `network.*`, `location.*`, `call.*`, `music.*` auto-forward to MQTT.
- All tests green; `uv run python scripts/quality.py ci` green.

---

## What's next (Plan 4)

`focus`, `diagnostics`, `screen`, `voice` — four integrations covering the cross-cutting signals (audio focus arbiter, event log + responsiveness watchdog + snapshot command, screen wake/idle/brightness, STT/TTS/voice-command). Focus must ship before music/call migrations since those depend on it.

---

*End of implementation plan.*
