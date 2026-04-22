"""Unit tests for network backend, PPP process, and manager."""

from __future__ import annotations

from unittest.mock import MagicMock, patch

from yoyopod.backends.network.modem import Sim7600Backend
from yoyopod.backends.network.ppp import PppProcess
from yoyopod.integrations.network.models import ModemPhase, ModemState, SignalInfo


def test_ppp_spawn_constructs_correct_command():
    """PppProcess.spawn should invoke pppd with the correct arguments."""
    with (
        patch("yoyopod.backends.network.ppp.shutil.which", return_value="pppd"),
        patch("yoyopod.backends.network.ppp.os.geteuid", return_value=0, create=True),
        patch(
            "yoyopod.backends.network.ppp.subprocess.run",
            return_value=MagicMock(returncode=0, stdout="default via 10.64.64.64 dev ppp0\n"),
        ),
        patch("subprocess.Popen") as mock_popen,
    ):
        mock_proc = MagicMock()
        mock_proc.poll.return_value = None
        mock_proc.pid = 12345
        mock_popen.return_value = mock_proc

        ppp = PppProcess(serial_port="/dev/ttyUSB3", apn="internet")
        assert ppp.spawn() is True

        args = mock_popen.call_args[0][0]
        assert "pppd" in args[0]
        assert "/dev/ttyUSB3" in args
        assert "defaultroute" in args
        assert "usepeerdns" in args
        assert mock_proc.pid == 12345


def test_ppp_spawn_uses_sbin_fallback_when_path_omits_pppd():
    """spawn() should still find pppd under /usr/sbin on minimal PATHs."""

    with (
        patch(
            "yoyopod.backends.network.ppp.shutil.which",
            side_effect=lambda candidate: "/usr/sbin/pppd" if candidate == "/usr/sbin/pppd" else None,
        ),
        patch("yoyopod.backends.network.ppp.os.geteuid", return_value=0, create=True),
        patch(
            "yoyopod.backends.network.ppp.subprocess.run",
            return_value=MagicMock(returncode=0, stdout="default via 10.64.64.64 dev ppp0\n"),
        ),
        patch("subprocess.Popen") as mock_popen,
    ):
        mock_proc = MagicMock()
        mock_proc.poll.return_value = None
        mock_proc.pid = 12345
        mock_popen.return_value = mock_proc

        ppp = PppProcess(serial_port="/dev/ttyUSB3", apn="internet")

        assert ppp.spawn() is True
        assert mock_popen.call_args[0][0][0] == "/usr/sbin/pppd"


def test_ppp_spawn_uses_sudo_wrapper_for_non_root_noauth() -> None:
    """spawn() should wrap pppd in sudo when the caller is not root."""

    def _which(candidate: str) -> str | None:
        if candidate in {"pppd", "/usr/sbin/pppd"}:
            return "/usr/sbin/pppd"
        if candidate in {"sudo", "/usr/bin/sudo"}:
            return "/usr/bin/sudo"
        return None

    with (
        patch("yoyopod.backends.network.ppp.shutil.which", side_effect=_which),
        patch("yoyopod.backends.network.ppp.os.geteuid", return_value=1000, create=True),
        patch(
            "yoyopod.backends.network.ppp.subprocess.run",
            return_value=MagicMock(returncode=0, stdout="default via 10.64.64.64 dev ppp0\n"),
        ),
        patch("subprocess.Popen") as mock_popen,
    ):
        mock_proc = MagicMock()
        mock_proc.poll.return_value = None
        mock_proc.pid = 12345
        mock_popen.return_value = mock_proc

        ppp = PppProcess(serial_port="/dev/ttyUSB3", apn="internet")

        assert ppp.spawn() is True
        assert mock_popen.call_args[0][0][:3] == ["/usr/bin/sudo", "-n", "/usr/sbin/pppd"]


def test_ppp_spawn_fails_without_sudo_for_non_root() -> None:
    """spawn() should fail early when noauth is needed but sudo is unavailable."""

    def _which(candidate: str) -> str | None:
        if candidate in {"pppd", "/usr/sbin/pppd"}:
            return "/usr/sbin/pppd"
        return None

    with (
        patch("yoyopod.backends.network.ppp.shutil.which", side_effect=_which),
        patch("yoyopod.backends.network.ppp.os.geteuid", return_value=1000, create=True),
        patch("yoyopod.backends.network.ppp.Path.exists", return_value=False),
        patch(
            "yoyopod.backends.network.ppp.subprocess.run",
            return_value=MagicMock(returncode=0, stdout="default via 10.64.64.64 dev ppp0\n"),
        ),
        patch("subprocess.Popen") as mock_popen,
    ):
        ppp = PppProcess(serial_port="/dev/ttyUSB3", apn="internet")

        assert ppp.spawn() is False
        mock_popen.assert_not_called()


def test_ppp_spawn_skips_default_route_and_peer_dns_when_wifi_is_already_primary() -> None:
    """spawn() should not let PPP take over global routing/DNS when another uplink is primary."""

    with (
        patch("yoyopod.backends.network.ppp.shutil.which", return_value="/usr/sbin/pppd"),
        patch("yoyopod.backends.network.ppp.os.geteuid", return_value=0, create=True),
        patch(
            "yoyopod.backends.network.ppp.subprocess.run",
            return_value=MagicMock(
                returncode=0,
                stdout="default via 192.168.178.1 dev wlan0 proto dhcp src 192.168.178.85 metric 50\n",
            ),
        ),
        patch("subprocess.Popen") as mock_popen,
    ):
        mock_proc = MagicMock()
        mock_proc.poll.return_value = None
        mock_proc.pid = 12345
        mock_popen.return_value = mock_proc

        ppp = PppProcess(serial_port="/dev/ttyUSB3", apn="internet")

        assert ppp.spawn() is True
        args = mock_popen.call_args[0][0]
        assert "defaultroute" not in args
        assert "usepeerdns" not in args


def test_ppp_kill_terminates_process():
    """PppProcess.kill should terminate then wait."""
    mock_proc = MagicMock()
    mock_proc.poll.return_value = None

    ppp = PppProcess(serial_port="/dev/ttyUSB3", apn="internet")
    ppp._process = mock_proc
    ppp.kill()

    mock_proc.terminate.assert_called_once()
    assert ppp._process is None


def test_ppp_is_alive_when_running():
    """is_alive should return True when pppd is running."""
    mock_proc = MagicMock()
    mock_proc.poll.return_value = None

    ppp = PppProcess(serial_port="/dev/ttyUSB3", apn="internet")
    ppp._process = mock_proc
    assert ppp.is_alive() is True


def test_ppp_is_alive_when_dead():
    """is_alive should return False when no process."""
    ppp = PppProcess(serial_port="/dev/ttyUSB3", apn="internet")
    assert ppp.is_alive() is False


class FakeAtCommands:
    """AT command double for backend tests."""

    def __init__(self) -> None:
        self.sim_ready = True
        self.registered = True
        self.signal = SignalInfo(csq=20)
        self.carrier = ("T-Mobile", "4G")
        self.gps_enabled = False
        self.calls: list[str] = []

    def ping(self) -> bool:
        self.calls.append("ping")
        return True

    def echo_off(self) -> None:
        self.calls.append("echo_off")

    def check_sim(self) -> bool:
        self.calls.append("check_sim")
        return self.sim_ready

    def get_signal_quality(self) -> SignalInfo:
        self.calls.append("get_signal_quality")
        return self.signal

    def get_carrier(self) -> tuple[str, str]:
        self.calls.append("get_carrier")
        return self.carrier

    def get_registration(self) -> bool:
        self.calls.append("get_registration")
        return self.registered

    def configure_pdp(self, apn: str) -> None:
        self.calls.append(f"configure_pdp:{apn}")

    def enable_gps(self) -> bool:
        self.calls.append("enable_gps")
        self.gps_enabled = True
        return True

    def hangup(self) -> None:
        self.calls.append("hangup")

    def radio_off(self) -> None:
        self.calls.append("radio_off")


class FakePpp:
    """PPP process double."""

    def __init__(self) -> None:
        self.alive = False
        self.link_up = True
        self.calls: list[str] = []

    def spawn(self) -> bool:
        self.calls.append("spawn")
        self.alive = True
        return True

    def wait_for_link(self, timeout: float = 30.0) -> bool:
        self.calls.append("wait_for_link")
        return self.link_up

    def is_alive(self) -> bool:
        return self.alive

    def kill(self) -> None:
        self.calls.append("kill")
        self.alive = False


def test_backend_probe_success():
    """probe should return True when modem responds to AT."""
    at = FakeAtCommands()
    ppp = FakePpp()
    backend = Sim7600Backend.__new__(Sim7600Backend)
    backend._at = at
    backend._ppp = ppp
    backend._gps = None
    backend._state = ModemState()
    backend._config = None

    assert backend.probe() is True
    assert "ping" in at.calls


def test_backend_init_modem_sequence():
    """init_modem should run the full startup sequence."""
    at = FakeAtCommands()
    ppp = FakePpp()
    backend = Sim7600Backend.__new__(Sim7600Backend)
    backend._at = at
    backend._ppp = ppp
    backend._gps = None
    backend._state = ModemState()

    class FakeConfig:
        gps_enabled = True
        apn = "internet"

    backend._config = FakeConfig()
    backend.init_modem()

    assert backend._state.phase == ModemPhase.REGISTERED
    assert backend._state.sim_ready is True
    assert backend._state.carrier == "T-Mobile"
    assert backend._state.network_type == "4G"
    assert backend._state.signal.bars == 3
    assert "enable_gps" in at.calls


def test_backend_start_ppp():
    """start_ppp should transition to ONLINE after link comes up."""
    at = FakeAtCommands()
    ppp = FakePpp()
    backend = Sim7600Backend.__new__(Sim7600Backend)
    backend._at = at
    backend._ppp = ppp
    backend._gps = None
    backend._state = ModemState(phase=ModemPhase.REGISTERED)

    class FakeConfig:
        apn = "internet"
        ppp_timeout = 30

    backend._config = FakeConfig()
    backend.start_ppp()

    assert backend._state.phase == ModemPhase.ONLINE
    assert "spawn" in ppp.calls
    assert "configure_pdp:internet" in at.calls


def test_backend_start_ppp_skips_blank_apn_configuration() -> None:
    """start_ppp() should not overwrite the modem PDP context with an empty APN."""

    at = FakeAtCommands()
    ppp = FakePpp()
    backend = Sim7600Backend.__new__(Sim7600Backend)
    backend._at = at
    backend._ppp = ppp
    backend._gps = None
    backend._state = ModemState(phase=ModemPhase.REGISTERED)

    class FakeConfig:
        apn = "   "
        ppp_timeout = 30

    backend._config = FakeConfig()
    backend.start_ppp()

    assert backend._state.phase == ModemPhase.ONLINE
    assert "spawn" in ppp.calls
    assert not any(call.startswith("configure_pdp:") for call in at.calls)


def test_backend_start_ppp_skips_null_apn_configuration() -> None:
    """start_ppp() should tolerate null APN values from config overlays."""

    at = FakeAtCommands()
    ppp = FakePpp()
    backend = Sim7600Backend.__new__(Sim7600Backend)
    backend._at = at
    backend._ppp = ppp
    backend._gps = None
    backend._state = ModemState(phase=ModemPhase.REGISTERED)

    class FakeConfig:
        apn = None
        ppp_timeout = 30

    backend._config = FakeConfig()
    backend.start_ppp()

    assert backend._state.phase == ModemPhase.ONLINE
    assert "spawn" in ppp.calls
    assert not any(call.startswith("configure_pdp:") for call in at.calls)


def test_backend_is_online_marks_dead_ppp_process_offline() -> None:
    """is_online() should reconcile stale ONLINE state when pppd has already exited."""

    at = FakeAtCommands()
    ppp = FakePpp()
    ppp.alive = False
    backend = Sim7600Backend.__new__(Sim7600Backend)
    backend._at = at
    backend._ppp = ppp
    backend._gps = None
    backend._state = ModemState(phase=ModemPhase.ONLINE)

    class FakeConfig:
        gps_enabled = False
        apn = "internet"

    backend._config = FakeConfig()

    assert backend.is_online() is False
    assert backend._state.phase == ModemPhase.REGISTERED
    assert backend._state.error == "PPP process exited"


def test_backend_is_online_marks_missing_ppp_interface_offline() -> None:
    """is_online() should drop back to REGISTERED when the PPP interface disappears."""

    at = FakeAtCommands()
    ppp = FakePpp()
    ppp.alive = True
    backend = Sim7600Backend.__new__(Sim7600Backend)
    backend._at = at
    backend._ppp = ppp
    backend._gps = None
    backend._state = ModemState(phase=ModemPhase.ONLINE)

    class FakeConfig:
        gps_enabled = False
        apn = "internet"

    backend._config = FakeConfig()

    with patch("yoyopod.backends.network.modem.Path.exists", return_value=False):
        assert backend.is_online() is False

    assert backend._state.phase == ModemPhase.REGISTERED
    assert backend._state.error == "PPP interface down"
