"""Power management foundation for YoyoPod."""

from yoyopy.power.backend import (
    PiSugarAutoTransport,
    PiSugarBackend,
    PiSugarTCPTransport,
    PiSugarUnixTransport,
    PowerBackend,
    PowerTransportError,
    build_pisugar_transport,
)
from yoyopy.power.events import (
    GracefulShutdownCancelled,
    GracefulShutdownRequested,
    LowBatteryWarningRaised,
    PowerAvailabilityChanged,
    PowerSnapshotUpdated,
)
from yoyopy.power.manager import PowerManager
from yoyopy.power.models import (
    BatteryState,
    PowerConfig,
    PowerDeviceInfo,
    PowerSnapshot,
    RTCState,
    ShutdownState,
)
from yoyopy.power.policies import PowerSafetyPolicy

__all__ = [
    "PowerBackend",
    "PowerTransportError",
    "PiSugarBackend",
    "PiSugarTCPTransport",
    "PiSugarUnixTransport",
    "PiSugarAutoTransport",
    "build_pisugar_transport",
    "PowerManager",
    "PowerConfig",
    "PowerDeviceInfo",
    "BatteryState",
    "RTCState",
    "ShutdownState",
    "PowerSnapshot",
    "PowerSnapshotUpdated",
    "PowerAvailabilityChanged",
    "LowBatteryWarningRaised",
    "GracefulShutdownRequested",
    "GracefulShutdownCancelled",
    "PowerSafetyPolicy",
]

