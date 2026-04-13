"""Unit tests for network backend, PPP process, and manager."""

from __future__ import annotations

import subprocess
from unittest.mock import MagicMock, patch

from yoyopy.network.ppp import PppProcess


def test_ppp_spawn_constructs_correct_command():
    """PppProcess.spawn should invoke pppd with the correct arguments."""
    with patch("subprocess.Popen") as mock_popen:
        mock_proc = MagicMock()
        mock_proc.poll.return_value = None
        mock_proc.pid = 12345
        mock_popen.return_value = mock_proc

        ppp = PppProcess(serial_port="/dev/ttyS0", apn="internet")
        assert ppp.spawn() is True

        args = mock_popen.call_args[0][0]
        assert "pppd" in args[0]
        assert "/dev/ttyS0" in args
        assert mock_proc.pid == 12345


def test_ppp_kill_terminates_process():
    """PppProcess.kill should terminate then wait."""
    mock_proc = MagicMock()
    mock_proc.poll.return_value = None

    ppp = PppProcess(serial_port="/dev/ttyS0", apn="internet")
    ppp._process = mock_proc
    ppp.kill()

    mock_proc.terminate.assert_called_once()
    assert ppp._process is None


def test_ppp_is_alive_when_running():
    """is_alive should return True when pppd is running."""
    mock_proc = MagicMock()
    mock_proc.poll.return_value = None

    ppp = PppProcess(serial_port="/dev/ttyS0", apn="internet")
    ppp._process = mock_proc
    assert ppp.is_alive() is True


def test_ppp_is_alive_when_dead():
    """is_alive should return False when no process."""
    ppp = PppProcess(serial_port="/dev/ttyS0", apn="internet")
    assert ppp.is_alive() is False
