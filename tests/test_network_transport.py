"""Unit tests for the UART serial transport and AT command layer."""

from __future__ import annotations

import threading
from unittest.mock import MagicMock, patch

from yoyopy.network.transport import SerialTransport, TransportError
from yoyopy.network.at_commands import AtCommandSet
from yoyopy.network.models import SignalInfo


class FakeTransport:
    """Minimal transport double for AT command tests."""

    def __init__(self) -> None:
        self.responses: dict[str, str] = {}
        self.sent: list[str] = []

    def send_command(self, command: str, timeout: float | None = None) -> str:
        self.sent.append(command)
        for prefix, response in self.responses.items():
            if command.strip().startswith(prefix):
                return response
        return "OK"


class FakeSerial:
    """Minimal pyserial double."""

    def __init__(self) -> None:
        self.is_open = True
        self._response = b"OK\r\n"
        self.written: list[bytes] = []

    def write(self, data: bytes) -> int:
        self.written.append(data)
        return len(data)

    def read_until(self, expected: bytes = b"\n", size: int | None = None) -> bytes:
        return self._response

    def readline(self) -> bytes:
        return self._response

    def read(self, size: int = 1) -> bytes:
        return self._response[:size]

    def reset_input_buffer(self) -> None:
        pass

    def close(self) -> None:
        self.is_open = False

    @property
    def in_waiting(self) -> int:
        return len(self._response)


def test_send_command_returns_response():
    """send_command should write AT command and return parsed response."""
    fake = FakeSerial()
    transport = SerialTransport.__new__(SerialTransport)
    transport._serial = fake
    transport._lock = threading.Lock()

    result = transport.send_command("AT")
    assert "OK" in result
    assert any(b"AT\r\n" in w for w in fake.written)


def test_send_command_raises_on_closed_port():
    """send_command should raise TransportError when port is closed."""
    transport = SerialTransport.__new__(SerialTransport)
    transport._serial = None
    transport._lock = threading.Lock()

    try:
        transport.send_command("AT")
        assert False, "Expected TransportError"
    except TransportError:
        pass


def test_parse_signal_quality():
    """get_signal_quality should parse AT+CSQ response into SignalInfo."""
    transport = FakeTransport()
    transport.responses["AT+CSQ"] = "+CSQ: 18,0\nOK"
    at = AtCommandSet(transport)
    info = at.get_signal_quality()
    assert info.csq == 18
    assert info.bars == 3


def test_check_sim_ready():
    """check_sim should return True when SIM is READY."""
    transport = FakeTransport()
    transport.responses["AT+CPIN?"] = "+CPIN: READY\nOK"
    at = AtCommandSet(transport)
    assert at.check_sim() is True


def test_check_sim_not_inserted():
    """check_sim should return False when SIM is missing."""
    transport = FakeTransport()
    transport.responses["AT+CPIN?"] = "+CME ERROR: 10"
    at = AtCommandSet(transport)
    assert at.check_sim() is False


def test_get_carrier():
    """get_carrier should parse AT+COPS? response."""
    transport = FakeTransport()
    transport.responses["AT+COPS?"] = '+COPS: 0,0,"T-Mobile",7\nOK'
    at = AtCommandSet(transport)
    carrier, network_type = at.get_carrier()
    assert carrier == "T-Mobile"
    assert network_type == "4G"


def test_get_registration_registered():
    """get_registration should detect home registration."""
    transport = FakeTransport()
    transport.responses["AT+CEREG?"] = "+CEREG: 0,1\nOK"
    at = AtCommandSet(transport)
    assert at.get_registration() is True


def test_get_registration_not_registered():
    """get_registration should detect unregistered state."""
    transport = FakeTransport()
    transport.responses["AT+CEREG?"] = "+CEREG: 0,0\nOK"
    at = AtCommandSet(transport)
    assert at.get_registration() is False


from yoyopy.network.gps import GpsReader


def test_gps_reader_query_with_fix():
    """GpsReader.query should return coordinates when GPS has a fix."""
    transport = FakeTransport()
    transport.responses["AT+CGPSINFO"] = (
        "+CGPSINFO: 4852.4300,N,00221.1300,E,130426,120000.0,35.0,0.5,\nOK"
    )
    reader = GpsReader(transport)
    coord = reader.query()
    assert coord is not None
    assert coord.lat > 0
    assert coord.lng > 0


def test_gps_reader_query_no_fix():
    """GpsReader.query should return None when no GPS fix."""
    transport = FakeTransport()
    transport.responses["AT+CGPSINFO"] = "+CGPSINFO: ,,,,,,,,\nOK"
    reader = GpsReader(transport)
    assert reader.query() is None


def test_gps_reader_enable():
    """GpsReader.enable should send AT+CGPS=1."""
    transport = FakeTransport()
    transport.responses["AT+CGPS=1"] = "OK"
    reader = GpsReader(transport)
    assert reader.enable() is True
    assert "AT+CGPS=1" in transport.sent
