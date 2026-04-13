"""GPS query and coordinate parsing for SIM7600G-H."""

from __future__ import annotations

from typing import TYPE_CHECKING

from loguru import logger

from yoyopy.network.at_commands import AtCommandSet

if TYPE_CHECKING:
    from yoyopy.network.models import GpsCoordinate


class GpsReader:
    """Query GPS position via AT commands on the SIM7600G-H."""

    def __init__(self, transport) -> None:
        self._at = AtCommandSet(transport)

    def enable(self) -> bool:
        """Enable the GPS engine on the modem."""
        return self._at.enable_gps()

    def disable(self) -> None:
        """Disable the GPS engine."""
        self._at.disable_gps()

    def query(self) -> GpsCoordinate | None:
        """Query current GPS fix. Returns None if no fix available."""
        return self._at.query_gps()
