"""pppd subprocess lifecycle manager."""

from __future__ import annotations

import subprocess

from loguru import logger


class PppProcess:
    """Spawn, monitor, and kill a pppd process for cellular data."""

    def __init__(self, serial_port: str, apn: str, baud_rate: int = 115200) -> None:
        self.serial_port = serial_port
        self.apn = apn
        self.baud_rate = baud_rate
        self._process: subprocess.Popen | None = None

    def spawn(self) -> bool:
        """Launch pppd for cellular data."""
        if self._process is not None and self._process.poll() is None:
            logger.warning("pppd already running (pid={})", self._process.pid)
            return True

        cmd = [
            "pppd",
            self.serial_port,
            str(self.baud_rate),
            "nodetach",
            "noauth",
            "defaultroute",
            "usepeerdns",
            "persist",
            "connect",
            f"chat -v '' AT OK 'ATD*99#' CONNECT",
        ]

        try:
            self._process = subprocess.Popen(
                cmd,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
            )
            logger.info("pppd spawned (pid={})", self._process.pid)
            return True
        except FileNotFoundError:
            logger.error("pppd binary not found")
            return False
        except Exception as exc:
            logger.error("Failed to spawn pppd: {}", exc)
            return False

    def is_alive(self) -> bool:
        """Return True when the pppd process is running."""
        return self._process is not None and self._process.poll() is None

    def kill(self) -> None:
        """Terminate the pppd process."""
        if self._process is None:
            return

        try:
            self._process.terminate()
            self._process.wait(timeout=5.0)
        except subprocess.TimeoutExpired:
            logger.warning("pppd did not exit after SIGTERM, sending SIGKILL")
            self._process.kill()
            self._process.wait(timeout=5.0)
        except Exception as exc:
            logger.error("Error killing pppd: {}", exc)
        finally:
            self._process = None

    def respawn(self) -> bool:
        """Kill and restart pppd."""
        self.kill()
        return self.spawn()
